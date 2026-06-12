# Goals

- Waypoint v3: additive path[] canonical position; synthesize from v2 on read; keep parentPhase/childPointer derived
- kbd_node_dir/kbd_current_node_dir resolvers in waypoint.sh; incremental script migration behind tests
- Arbitrary-depth kbd-new-child (drop depth-1 restriction) + child scope.json + handoff-in.md
- kbd-child-exit skill: handoff-out, progress rollup up the ancestor chain, pop path
- check-child-scope.sh PreToolUse hook enforcing child scope.json (advisory, canonicalized paths)
