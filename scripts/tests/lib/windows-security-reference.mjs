/**
 * A TEST-ONLY reference reader for Windows security descriptors.
 *
 * Production reads a descriptor through `prometheus-exec inspect-file-security`,
 * which calls GetNamedSecurityInfoW directly. That binary must be built, and a
 * fixture that could only run where it had been built would leave the predicate
 * itself untested on every other host.
 *
 * This reader produces the SAME report shape from the SAME operating system
 * data, via .NET's `System.Security.AccessControl`. The descriptors it returns
 * are real -- read from real files with real access control lists, including
 * ones this fixture created with `icacls` -- so the predicate is exercised
 * against genuine Windows security data, not a hand-written fake.
 *
 * It resolves every principal to a string SID, exactly as the Rust reader does,
 * because a report that carried display names would let a predicate accidentally
 * compare them and pass on an English host while failing on a German one.
 *
 * Two deliberate limitations, both fail-safe:
 *
 *   * A NULL DACL is reported as absent only when .NET surfaces no rule
 *     collection at all. A file with a genuine NULL DACL is not something this
 *     fixture creates, and the production reader distinguishes the case
 *     properly via GetSecurityDescriptorControl.
 *   * Only the DACL is read. Audit entries live in the SACL and cannot grant
 *     access, so they are outside the predicate's question.
 */

import { spawnSync } from 'node:child_process';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const SCRIPT = String.raw`
param([string]$Target)
$ErrorActionPreference = 'Stop'
$acl = Get-Acl -LiteralPath $Target
$sidType = [System.Security.Principal.SecurityIdentifier]
$owner = $acl.GetOwner($sidType).Value
$group = $null
try { $group = $acl.GetGroup($sidType).Value } catch { $group = $null }
$rules = @()
$inherited = 0
foreach ($rule in $acl.GetAccessRules($true, $true, $sidType)) {
  if ($rule.IsInherited) { $inherited += 1 }
  $rules += [ordered]@{
    sid        = $rule.IdentityReference.Value
    kind       = if ($rule.AccessControlType -eq 'Allow') { 'allow' } else { 'deny' }
    inherited  = [bool]$rule.IsInherited
    accessMask = [int]$rule.FileSystemRights
  }
}
$report = [ordered]@{
  schemaVersion    = 1
  model            = 'windows-security-descriptor'
  path             = (Resolve-Path -LiteralPath $Target).Path
  processOwnerSid  = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
  ownerSid         = $owner
  groupSid         = $group
  daclPresent      = $true
  daclProtected    = [bool]$acl.AreAccessRulesProtected
  inheritedAceCount = $inherited
  aces             = @($rules)
}
$report | ConvertTo-Json -Depth 5 -Compress
`;

// `powershell -Command <string>` cannot take a param() block plus arguments;
// only `-File` binds them. The script is therefore materialized once per run.
let scriptFile = null;
function scriptPath() {
  if (scriptFile && fs.existsSync(scriptFile)) return scriptFile;
  scriptFile = path.join(
    os.tmpdir(),
    `prometheus-security-reference-${crypto.randomBytes(6).toString('hex')}.ps1`
  );
  fs.writeFileSync(scriptFile, SCRIPT, 'utf8');
  return scriptFile;
}

/** Read `filePath`'s descriptor. Returns the `{descriptor, error}` inspector shape. */
export function referenceWindowsInspector(filePath) {
  const result = spawnSync(
    'powershell.exe',
    ['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', scriptPath(), '-Target', filePath],
    { encoding: 'utf8', shell: false }
  );
  if (result.error || result.status !== 0) {
    return {
      descriptor: null,
      error: result.error?.message ?? `powershell exited ${result.status}: ${(result.stderr ?? '').trim()}`,
    };
  }
  try {
    const parsed = JSON.parse(result.stdout);
    // ConvertTo-Json collapses a single-element array to a bare object.
    if (parsed.aces && !Array.isArray(parsed.aces)) parsed.aces = [parsed.aces];
    return { descriptor: parsed, error: null };
  } catch (error) {
    return { descriptor: null, error: `reference reader emitted invalid JSON: ${error.message}` };
  }
}

/** Current process token user SID, used to build the expected trustee set. */
export function currentUserSid() {
  const result = spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-NonInteractive',
      '-Command',
      '[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value',
    ],
    { encoding: 'utf8', shell: false }
  );
  if (result.error || result.status !== 0) return null;
  return result.stdout.trim() || null;
}

/** Apply an access control list with `icacls`. Fixture setup only. */
export function icacls(args) {
  const result = spawnSync('icacls.exe', args, { encoding: 'utf8', shell: false });
  return { status: result.status, stdout: result.stdout ?? '', stderr: result.stderr ?? '' };
}
