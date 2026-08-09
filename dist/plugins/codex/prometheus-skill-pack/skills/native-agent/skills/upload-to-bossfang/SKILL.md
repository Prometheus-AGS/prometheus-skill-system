---
name: upload-to-bossfang
description: Upload a packaged .lf-skill.zip to a running LibreFang/bossfang instance via its /skills/install REST endpoint. Validates the URL with a deny-by-default SSRF guard (rejects private IPs, non-http schemes, and unallowlisted hosts unless --insecure is passed for explicit local-dev use). Auto-discovers the zip in cwd, redacts the BOSSFANG_TOKEN bearer auth from all logs, and verifies the install by GETting the skill manifest after upload. Use after `forge package-librefang` produces a zip and you want to deploy it to a remote bossfang.
license: MIT
version: '1.0.0'
authors:
  - Prometheus AGS
metadata:
  category: process
  tags: [librefang, bossfang, deploy, upload, ssrf-guard, security-reviewed]
  slash_command: '/upload-to-bossfang'
  requires_security_review: true
  parent_skill: native-agent
triggers:
  keywords:
    - upload to bossfang
    - upload to librefang
    - install wasm skill
    - deploy lf-skill
    - bossfang install
  semantic: >
    Push a .lf-skill.zip artifact to a remote LibreFang Agent OS via its
    REST /skills/install endpoint, with SSRF protections.
---

# /upload-to-bossfang

POSTs a `.lf-skill.zip` to a LibreFang/bossfang instance's `/skills/install`
endpoint. Defense-in-depth SSRF guards prevent the slash-command from being
used to scan internal networks or hit cloud-metadata services.

## Usage

```
/upload-to-bossfang <url> [--zip <path>] [--insecure]
```

| Flag | Purpose |
|---|---|
| `<url>` | Required. Bossfang base URL (e.g. `https://bossfang.example.com`). Must be `http(s)://`, must NOT resolve to a private IP, and host:port must appear in `~/.config/prometheus-skill-pack/bossfang-allowlist.toml` unless `--insecure` is set. |
| `--zip <path>` | Optional. Path to the `.lf-skill.zip`. If omitted, the script searches cwd for exactly one `*.lf-skill.zip`. |
| `--insecure` | Skip the allowlist check (still enforces all other guards). Required to push to ad-hoc public URLs and to localhost. |

Auth: if `BOSSFANG_TOKEN` env var is set, sent as `Authorization: Bearer $TOKEN`.
The token is **never** echoed to stdout/stderr/logs, even on curl failure.

## SSRF Guard Pipeline

The script runs a deny-by-default pipeline before any network call:

1. **Scheme** — only `http://` and `https://` are allowed. Reject `file://`,
   `gopher://`, `dict://`, `ldap://`, `ftp://`, `jar://`, `php://`, `data:`,
   `javascript:`, `blob:`.
2. **Hostname extraction** — standard URL parse. Reject if the URL has
   embedded credentials (`http://user:pass@host`).
3. **DNS resolution** — resolve once via `getent ahosts <host>`. Reject if
   ANY A/AAAA record is in:
   - `127.0.0.0/8` (loopback)
   - `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16` (RFC 1918)
   - `169.254.0.0/16` (link-local, blocks AWS/GCP metadata 169.254.169.254)
   - `100.64.0.0/10` (CGNAT)
   - `0.0.0.0/8`, `224.0.0.0/4`, `240.0.0.0/4` (reserved/multicast)
   - `::1`, `fc00::/7`, `fe80::/10` (IPv6 loopback/private/link-local)
4. **Allowlist** — read `~/.config/prometheus-skill-pack/bossfang-allowlist.toml`
   and require `<host>:<port>` match. `--insecure` skips this step.
5. **Special case** — `http://localhost:4545` / `http://127.0.0.1:4545`
   require `--insecure` (LibreFang's default port is 4545).

The script then invokes `curl` with explicit defenses:

- `--proto =https,http` — refuses any other scheme even on redirects.
- `--max-redirs 0` — refuses to follow redirects (would defeat IP validation).
- `--resolve <host>:<port>:<validated_ip>` — pins the IP curl uses to the one
  the SSRF check validated. Defeats DNS rebinding.
- `--connect-timeout 10 --max-time 60` — no slowloris.
- `--fail-with-body` — non-2xx body is shown but exit is non-zero.

## Allowlist Format

Default location: `~/.config/prometheus-skill-pack/bossfang-allowlist.toml`.

```toml
# One entry per allowed bossfang instance.
[[allowed]]
host = "bossfang.example.com"
port = 443
notes = "Production fleet — owned by the SRE team"

[[allowed]]
host = "staging-bossfang.example.com"
port = 443

# `--insecure` localhost is built in and does not need a TOML entry.
```

## Behavior

```
1. Resolve <zip>:
   - If --zip given: validate the file exists.
   - Else: glob *.lf-skill.zip in cwd. Fail if 0 or >1 matches.

2. Extract skill name from zip:
   - Use `unzip -p <zip> skill.toml` capped at 64 KB (prevents zip bombs).
   - Parse with `tomlq` or fallback to grep '^name = "'.
   - Sanitize: skill name MUST match ^[a-z0-9][a-z0-9-]+$ (no path traversal).

3. Run SSRF pipeline above.

4. POST <validated-url>/skills/install with the zip body.

5. POST <validated-url>/skills/reload.

6. GET <validated-url>/skills/<sanitized-skill-name> and pretty-print.

7. Print summary: skill name, runtime type, file count, install timestamp.
```

## Failure Modes

| Exit code | Meaning |
|---|---|
| 0 | Success — skill installed, reloaded, and verified |
| 1 | Generic error |
| 2 | URL validation failed (SSRF guard tripped) |
| 3 | Allowlist rejected the host (use --insecure to bypass after review) |
| 4 | Zip not found, or multiple zips present in cwd |
| 5 | Zip schema invalid (no skill.toml, or skill name fails regex) |
| 6 | curl upload failed (HTTP non-2xx; body shown without auth header) |
| 7 | Verification GET failed (skill installed but not visible) |

## Security Posture

This skill MUST be reviewed by `security-reviewer` before any change to its
SSRF pipeline. The validation is the only thing standing between an
internal-network probe and a "user-supplied URL" arg surface.

See [`references/threat-model.md`](references/threat-model.md) for the full
threat model produced during change-005.

## Reference Files

- [`scripts/upload.sh`](scripts/upload.sh) — the actual implementation.
- [`references/threat-model.md`](references/threat-model.md) — threat model.
- [`references/bossfang-allowlist.example.toml`](references/bossfang-allowlist.example.toml) — example config.
