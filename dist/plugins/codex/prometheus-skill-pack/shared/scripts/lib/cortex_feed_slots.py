"""Nonblocking, OS-owned capacity for optional Cortex feeder/server pairs.

Slot files are permanent inode names, not PID or age-based leases. The open
description is inherited by the feeder and its server so either live process
retains capacity, even if the writer or feeder crashes. Never unlink a slot.
"""
from __future__ import annotations

import errno
import os
import re
import stat
from pathlib import Path

try:
    import fcntl
except ImportError:
    fcntl = None

DEFAULT_LIMIT = 4
SLOT_NAME = re.compile(r"slot-([0-9]+)\.lock\Z")


class Lease:
    def __init__(self, descriptor: int):
        self.descriptor = descriptor

    def close(self) -> None:
        if self.descriptor >= 0:
            os.close(self.descriptor)
            self.descriptor = -1


def configured_limit() -> tuple[int, str | None]:
    raw = os.environ.get("PROMETHEUS_CORTEX_MAX_FEEDERS", str(DEFAULT_LIMIT))
    if re.fullmatch(r"[0-9]+", raw):
        try:
            return int(raw), None
        except ValueError:
            pass
    return DEFAULT_LIMIT, "invalid-feeder-limit; using default 4"


def pool_root() -> Path:
    queue = Path(os.environ.get("PROMETHEUS_LEARNING_QUEUE",
                                str(Path.home() / ".prometheus" / "learning-queue")))
    return queue.expanduser().resolve() / "cortex-feeders"


def _open(path: Path) -> int:
    fd = os.open(path, os.O_RDWR | os.O_CREAT | getattr(os, "O_NOFOLLOW", 0), 0o600)
    if not stat.S_ISREG(os.fstat(fd).st_mode):
        os.close(fd)
        raise OSError("feeder lease is not a regular file")
    return fd


def _lock(fd: int) -> bool:
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        return True
    except OSError as error:
        if error.errno in (errno.EAGAIN, errno.EACCES):
            return False
        raise


def acquire() -> tuple[Lease | None, dict]:
    limit, diagnostic = configured_limit()
    status = {"status": "failed", "reason": "admission-failed", "limit": limit}
    if diagnostic:
        status["diagnostic"] = diagnostic
    if limit == 0:
        status.update(status="disabled", reason="feeder-limit-zero")
        return None, status
    if fcntl is None or os.name != "posix":
        # No unsafe PID/age fallback: the mirror is optional; primary writes
        # remain available when inherited OS lock ownership is unsupported.
        status["reason"] = "inherited-locks-unavailable"
        return None, status
    admission = -1
    available: list[tuple[int, int]] = []
    try:
        root = pool_root()
        root.mkdir(mode=0o700, parents=True, exist_ok=True)
        admission = _open(root / "admission.lock")
        if not _lock(admission):
            status.update(status="saturated", reason="admission-busy")
            return None, status
        indexes = sorted({int(match.group(1)) for entry in root.iterdir()
                          if (match := SLOT_NAME.fullmatch(entry.name))})
        live = 0
        for index in indexes:
            fd = _open(root / f"slot-{index}.lock")
            try:
                if _lock(fd):
                    available.append((index, fd))
                else:
                    live += 1
                    os.close(fd)
            except OSError:
                os.close(fd)
                raise
        # Count all live slots, including those from a previously larger
        # limit. Decreasing the limit never admits above the new capacity.
        if live >= limit:
            status.update(status="saturated", reason="capacity-full")
            return None, status
        if available:
            _, descriptor = available.pop(0)
        else:
            index = 0
            occupied = set(indexes)
            while index in occupied:
                index += 1
            descriptor = _open(root / f"slot-{index}.lock")
            try:
                locked = _lock(descriptor)
            except OSError:
                os.close(descriptor)
                raise
            if not locked:
                os.close(descriptor)
                status.update(status="saturated", reason="capacity-raced")
                return None, status
        status.update(status="accepted", reason="capacity-acquired")
        return Lease(descriptor), status
    except (OSError, ValueError, OverflowError):
        return None, status
    finally:
        for _, descriptor in available:
            os.close(descriptor)
        if admission >= 0:
            os.close(admission)


def inherit(descriptor: int) -> Lease:
    if fcntl is None or descriptor < 3 or not stat.S_ISREG(os.fstat(descriptor).st_mode):
        raise OSError("invalid inherited feeder lease")
    # The inherited open description already owns the lock; this also rejects
    # an unrelated descriptor locked by another process.
    if not _lock(descriptor):
        raise OSError("inherited feeder lease is not owned")
    return Lease(descriptor)
