# /upload-to-bossfang Threat Model

> Produced during change-005 of phase-compliance-and-power-multiplier.
> Reviewed by `security-reviewer` before merge.

## Trust Boundaries

```
┌──────────────────┐    user-supplied URL + zip    ┌─────────────────┐
│ Claude Code user │ ────────────────────────────▶ │ upload.sh       │
└──────────────────┘                               │ (script)        │
                                                   └─────────────────┘
                                                            │ HTTP
                                                            ▼
                                                   ┌─────────────────┐
                                                   │ bossfang server │
                                                   │ (untrusted)     │
                                                   └─────────────────┘
```

- **Trusted**: the user (their slash-command input) AND the local filesystem
  scope (`~/.config/`, cwd, `~/.cache/`).
- **Untrusted**: the URL the user typed (could be wrong/malicious),
  the bossfang server's responses (could attempt redirects, return malicious
  data), the zip file (could be a zip bomb or have a malicious skill.toml),
  the DNS infrastructure (could rebind), the auth token (high-value secret).
- **Adversary scope**: a remote attacker who can (a) trick the user into
  pasting a URL, or (b) place a malicious zip on disk, or (c) compromise
  DNS in the user's resolution path.

## Attack Surface

| Surface | Input | Sanitization |
|---|---|---|
| `URL` arg | user CLI | scheme allowlist, no embedded credentials, hostname regex, port range, DNS-resolved IP allowlist, optional bossfang-allowlist.toml |
| `--zip <path>` arg | user CLI | path must exist, must be regular file, must not start with `-` |
| `BOSSFANG_TOKEN` env | parent shell | never logged; passed via `curl --header @<file>` (mode 0600); cleaned up on exit |
| `bossfang-allowlist.toml` | filesystem | mode-checked (refuses if group/world-writable); naive TOML scan with no eval; not executed |
| zip body | filesystem | `unzip -p` capped at 64 KB to prevent zip-bomb OOM |
| `skill.toml` inside zip | bossfang author | extracted name regex `^[a-z0-9][a-z0-9-]+$` (rejects path traversal in URL construction) |
| HTTP responses from bossfang | server | `--no-location` + `--max-redirs 0` (no redirect-following); `--proto =http,https` (no protocol switch); `--resolve` pin (no DNS rebinding) |

## Threat → Mitigation Matrix

| Threat | Severity | Mitigation in upload.sh |
|---|---|---|
| **DNS rebinding** | CRITICAL | `getent`/`dig` resolves once, the resulting IP is pinned in `--resolve` so curl never re-resolves |
| **30x redirect to internal host** | CRITICAL | `--no-location` + `--max-redirs 0` + `--proto-redir =http,https` |
| **TOML allowlist injection** | LOW | Refuses to read group/world-writable allowlist; naive parse, no eval |
| **Token leakage via argv** | HIGH | Token never on argv; written to `mktemp` mode 0600 file, passed as `--header @<file>` |
| **Token leakage via `set -x`** | HIGH | `set +x` at script top defeats parent-inherited xtrace |
| **Token leakage via curl error output** | HIGH | curl stderr piped to a file then `sed` redacts `Authorization:` lines before showing |
| **Token leakage on script crash** | HIGH | EXIT trap unlinks the header file even on `set -e` failure |
| **Argument injection (URL flags)** | HIGH | URL must match scheme prefix; positional only after explicit flag parsing |
| **Argument injection (zip path)** | HIGH | Zip path rejected if it starts with `-`; passed quoted via `--data-binary @"$ZIP"` |
| **TOCTOU between IP check and curl** | CRITICAL | Eliminated by `--resolve` (curl uses the pre-validated IP) |
| **Zip bomb (skill.toml extraction)** | MEDIUM | `dd bs=65536 count=1` caps decompressed read at 64 KB |
| **Zip bomb (full upload)** | MEDIUM | Out of scope — bossfang is responsible for zip-content limits at install time |
| **Cloud metadata leak (169.254.169.254)** | CRITICAL | `169.254.0.0/16` is in the deny-list IP CIDR check |
| **Localhost dev confusion** | LOW | `--insecure` ALONE allows `localhost:4545`; production allowlist works without `--insecure` |
| **Malicious skill name → URL injection** | HIGH | Extracted name validated against `^[a-z0-9][a-z0-9-]+$` regex before use |
| **Compromised allowlist file** | MEDIUM | Mode check refuses group/world-writable; doc recommends `chmod 600` |

## What This Skill Does NOT Defend Against

- **A user who types the wrong URL with `--insecure`**. The flag exists because
  developers genuinely need to push to ad-hoc URLs (ngrok tunnels, fresh
  staging instances). The script makes the bypass explicit; documentation
  reinforces "review before using `--insecure`".
- **A bossfang server that lies about install success**. We GET back the
  manifest after upload, but the server controls that response. If the
  bossfang admin is malicious, no client-side check helps — that's a deeper
  trust failure.
- **A compromised local user account.** If the attacker has the user's
  shell, they have `BOSSFANG_TOKEN` directly; the script's defenses are
  beside the point. Token rotation discipline lives outside this skill.

## Reviewer Recommendations Adopted

The change-005 security review (in-process) flagged 7 issues; all were
addressed before this skill shipped:

1. ✅ DNS rebinding → `--resolve` pin
2. ✅ Redirect following → `--no-location` + `--max-redirs 0` + `--proto-redir`
3. ✅ Allowlist file permissions → mode check at runtime
4. ✅ Token leakage → `--header @<file>` + `set +x` + EXIT trap unlink
5. ✅ Argument injection → quote variables + reject `-` prefix on zip path
6. ✅ TOCTOU → resolved by #1
7. ✅ Zip bomb → `dd bs=65536 count=1` cap on `unzip -p`

Plus the existing skill-name regex (which was already in the design) addresses
URL-construction injection from a malicious zip.

## Production Hardening (Future Work)

For a fleet that uses this routinely, the bash script should be replaced by
a Rust `forge upload-bossfang` subcommand that:

- Uses a real TOML parser (rejects malformed allowlists rather than failing open)
- Issues a HEAD request first to validate the bossfang version before uploading
- Emits structured JSON for CI integration
- Supports OAuth device flow rather than a static bearer token

That work is queued under `tools/forge-rs/.forge/changes/forge-package-librefang/`
for the `phase-librefang-wasm-onramp` phase.
