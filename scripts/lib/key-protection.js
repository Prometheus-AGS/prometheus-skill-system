/**
 * Owner-only protection for private signing key material.
 *
 * ONE GUARANTEE, TWO ASSERTIONS
 *
 * The guarantee is unchanged from the POSIX-only original: the private key is
 * readable only by its owner and by principals the host cannot exclude. What
 * changes is that the guarantee is now asserted by a mechanism the host can
 * actually express.
 *
 * The old gate was `fs.statSync(key).mode & 0o777 === 0o600`. On Windows libuv
 * synthesizes st_mode from FILE_ATTRIBUTE_READONLY alone, so a stat returns
 * 0o444 or 0o666 and NEVER 0o600, and `fs.chmodSync` only toggles the read-only
 * attribute. The gate was not merely wrong there; it was unsatisfiable, so the
 * installer could never complete twice on the same Windows host.
 *
 * The Windows assertion is Win32-OpenSSH's, adopted deliberately rather than
 * invented: owner SID equals the process token's user SID, the DACL is
 * PROTECTED (so nothing is inherited), and every remaining trustee is the owner,
 * LocalSystem, or BUILTIN\Administrators. The last two are not a concession —
 * both can take ownership of any object regardless of its DACL, so excluding
 * them buys nothing and breaks backup and anti-malware software.
 *
 * TRUSTEES ARE COMPARED AS SIDs, NEVER NAMES
 *
 * "Administrators" is "Administratoren" on a German host and "Администраторы" on
 * a Russian one. Name comparison is the single most common defect in hand-rolled
 * versions of this check. Both the inspector and this predicate speak SIDs only.
 *
 * REMEDIATION IS REPORTED, NEVER APPLIED
 *
 * A key whose ACL is wrong may be a misconfiguration or may be the visible edge
 * of a compromise. Silently repairing it destroys the only evidence that
 * anything happened, so the predicate returns the exact `icacls` invocation and
 * the caller prints it.
 */

import fs from 'node:fs';
import { spawnSync } from 'node:child_process';

/** Principals a Windows host cannot meaningfully exclude from a file it owns. */
export const UNAVOIDABLE_TRUSTEE_SIDS = Object.freeze([
  'S-1-5-18', // LocalSystem
  'S-1-5-32-544', // BUILTIN\Administrators
]);

export const SECURITY_INSPECTOR_SCHEMA_VERSION = 1;

function verdict(ok, reason, detail, remediation) {
  return { ok, reason, detail, remediation };
}

/**
 * POSIX assertion: mode 0600 and ownership by the running user.
 *
 * Ownership was NOT checked by the original gate. A 0600 file owned by someone
 * else is unreadable by this process, so the install would fail later with a
 * confusing EACCES; asserting it here reports the real condition.
 */
export function evaluatePosixKeyProtection({ keyPath, stat, processUid }) {
  const mode = stat.mode & 0o777;
  if (mode !== 0o600) {
    return verdict(
      false,
      'POSIX_MODE',
      `mode is ${mode.toString(8).padStart(4, '0')}, expected 0600`,
      `chmod 600 ${JSON.stringify(keyPath)}`
    );
  }
  if (processUid !== null && stat.uid !== processUid) {
    return verdict(
      false,
      'POSIX_OWNER',
      `owner uid ${stat.uid} is not the running uid ${processUid}`,
      `chown ${processUid} ${JSON.stringify(keyPath)}`
    );
  }
  return verdict(true, 'OK', `mode 0600, owner uid ${stat.uid}`, null);
}

function icaclsRemediation(keyPath, ownerSid) {
  // `*<SID>` is the locale-independent principal form. Spelling the trustees as
  // names here would produce a command that fails on a non-English host, which
  // is the same defect the predicate itself avoids.
  const grants = [ownerSid, ...UNAVOIDABLE_TRUSTEE_SIDS]
    .map(sid => `/grant:r "*${sid}:F"`)
    .join(' ');
  return `icacls "${keyPath}" /inheritance:r ${grants}`;
}

/**
 * Windows assertion over a security descriptor produced by the inspector.
 *
 * `descriptor` is data, not a decision: every judgement is made here so the
 * inspector stays a dumb reader that can be replaced or re-implemented.
 */
export function evaluateWindowsKeyProtection({ keyPath, descriptor }) {
  if (
    descriptor?.schemaVersion !== SECURITY_INSPECTOR_SCHEMA_VERSION ||
    descriptor.model !== 'windows-security-descriptor' ||
    typeof descriptor.ownerSid !== 'string' ||
    typeof descriptor.processOwnerSid !== 'string' ||
    typeof descriptor.daclPresent !== 'boolean' ||
    typeof descriptor.daclProtected !== 'boolean' ||
    !Number.isInteger(descriptor.inheritedAceCount) ||
    !Array.isArray(descriptor.aces)
  ) {
    return verdict(
      false,
      'UNSUPPORTED_DESCRIPTOR',
      'security descriptor report is missing or malformed',
      null
    );
  }
  const ownerSid = descriptor.ownerSid;
  if (ownerSid !== descriptor.processOwnerSid) {
    return verdict(
      false,
      'OWNER_MISMATCH',
      `owner ${ownerSid} is not the process token user ${descriptor.processOwnerSid}`,
      `takeown /f "${keyPath}" && ${icaclsRemediation(keyPath, descriptor.processOwnerSid)}`
    );
  }
  if (!descriptor.daclPresent) {
    // A NULL DACL grants everyone full control. This is the worst case and is
    // distinct from an empty DACL, which denies everyone.
    return verdict(
      false,
      'DACL_ABSENT',
      'the file has no discretionary access control list, which grants full control to everyone',
      icaclsRemediation(keyPath, ownerSid)
    );
  }
  if (!descriptor.daclProtected || descriptor.inheritedAceCount > 0) {
    return verdict(
      false,
      'DACL_INHERITED',
      descriptor.daclProtected
        ? `access control list is protected but still carries ${descriptor.inheritedAceCount} inherited entr${descriptor.inheritedAceCount === 1 ? 'y' : 'ies'}`
        : 'access control list is not protected against inheritance, so its effective trustee set is unbounded',
      icaclsRemediation(keyPath, ownerSid)
    );
  }
  const allowed = new Set([ownerSid, ...UNAVOIDABLE_TRUSTEE_SIDS]);
  const unexpected = descriptor.aces
    .filter(ace => !ace.inherited)
    .map(ace => ace.sid)
    .filter(sid => !allowed.has(sid));
  if (unexpected.length) {
    return verdict(
      false,
      'UNEXPECTED_TRUSTEE',
      `access control list grants unexpected trustee${unexpected.length === 1 ? '' : 's'}: ${[...new Set(unexpected)].join(', ')}`,
      icaclsRemediation(keyPath, ownerSid)
    );
  }
  return verdict(
    true,
    'OK',
    `owner ${ownerSid}, protected access control list, ${descriptor.aces.length} owner/system/administrator entries`,
    null
  );
}

/**
 * Build an inspector backed by `prometheus-exec inspect-file-security`.
 *
 * Node exposes no ACL API at all, and `icacls` output is localized, so parsing
 * it would reintroduce exactly the name-comparison defect this module exists to
 * avoid. The read is delegated to a binary that calls GetNamedSecurityInfoW and
 * emits SIDs.
 */
export function prometheusExecInspector(binary) {
  return function inspect(keyPath) {
    const result = spawnSync(
      binary,
      ['inspect-file-security', '--path', keyPath, '--format', 'json'],
      { encoding: 'utf8', shell: false }
    );
    if (result.error || result.status !== 0) {
      return {
        descriptor: null,
        error:
          result.error?.message ??
          `${binary} exited ${result.status}: ${(result.stderr ?? '').trim()}`,
      };
    }
    try {
      return { descriptor: JSON.parse(result.stdout), error: null };
    } catch (error) {
      return { descriptor: null, error: `inspector emitted invalid JSON: ${error.message}` };
    }
  };
}

/** First usable inspector binary, or null when none is installed. */
export function resolveSecurityInspector({ env = process.env } = {}) {
  const candidates = [env.PROMETHEUS_EXEC_BIN, 'prometheus-exec'].filter(Boolean);
  for (const candidate of candidates) {
    const probe = spawnSync(candidate, ['--version'], { encoding: 'utf8', shell: false });
    if (!probe.error && probe.status === 0) return prometheusExecInspector(candidate);
  }
  return null;
}

const MISSING_INSPECTOR_REMEDIATION = [
  'install the signed inspector and re-run, or point the installer at an existing build:',
  '  cargo build --release -p prometheus-exec && export PROMETHEUS_EXEC_BIN=<path to prometheus-exec>',
].join('\n    ');

/**
 * Assert owner-only protection for `keyPath`, dispatching on PROBED capability
 * rather than on `process.platform`.
 *
 * `capabilities.posixModes` is the empirical question "does this volume record
 * and report POSIX permission bits", which is the only thing that makes the
 * mode assertion meaningful. A volume that cannot represent 0600 gets the
 * security-descriptor assertion instead, and a host that can offer neither
 * fails closed: an unassertable guarantee is not a satisfied one.
 */
export function assertKeyProtection(keyPath, { capabilities, inspect = undefined }) {
  if (capabilities.posixModes) {
    const stat = fs.statSync(keyPath);
    const processUid = typeof process.getuid === 'function' ? process.getuid() : null;
    return evaluatePosixKeyProtection({ keyPath, stat, processUid });
  }
  const inspector = inspect ?? resolveSecurityInspector();
  if (!inspector) {
    return verdict(
      false,
      'NO_SECURITY_INSPECTOR',
      'this volume cannot represent POSIX permission bits and no security-descriptor inspector is available, so owner-only protection cannot be asserted',
      MISSING_INSPECTOR_REMEDIATION
    );
  }
  const { descriptor, error } = inspector(keyPath);
  if (error) {
    return verdict(false, 'INSPECTOR_FAILED', error, MISSING_INSPECTOR_REMEDIATION);
  }
  return evaluateWindowsKeyProtection({ keyPath, descriptor });
}
