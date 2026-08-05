# Verification — kde-001

## Gates

1. `bash -n scripts/install-kimi-desktop-plugin.sh` and `/bin/bash -n` both pass
   (launchd path requires bash 3.2 compatibility).
2. `bash scripts/install-kimi-desktop-plugin.sh` exits 0.
3. Generated manifest parses as strict JSON. It contains `mcpServers` **only in
   the t1-positive branch**; in the t1-negative branch the correct state is NO
   `mcpServers` field plus a recorded negative result. Gate 3 passes in both
   branches — it fails only if the manifest is malformed, or if a field was
   emitted that t1 showed cannot work.
4. Package reports 145 skills; every skill dir has a `SKILL.md`.
5. No Mach-O binary anywhere in the package.
6. `npm run validate` → 145 skills, 0 errors.

## Decisive evidence

The change is only complete when a tool from at least one declared server is
**observably usable inside Kimi Desktop**. A manifest containing `mcpServers` is
not evidence — the field can be present and inert, which is exactly the failure
mode that made 149 slash commands silently do nothing.

If t1 shows loopback URLs are refused, the correct outcome is a recorded
negative result and NO manifest change.
