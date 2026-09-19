# Verification — change-cpc-014-tui-json-command-leak

Run locally after the edit batch. Each command must exit 0.

## Verify block

```bash
# 1. tui.json is no longer a registration target; polluted installs get repaired
bash scripts/register-slash-commands.sh
python3 -c "import json,os; d=json.load(open(os.path.expanduser('~/.config/opencode/tui.json'))); assert 'command' not in d, 'command key leaked into tui.json'; print('tui.json clean, keys:', list(d.keys()))"

# 2. command registration still lands in both opencode.json files
python3 -c "import json,os; h=os.path.expanduser; a=json.load(open(h('~/.config/opencode/opencode.json'))).get('command',{}); b=json.load(open(h('~/.opencode/opencode.json'))).get('command',{}); assert len(a)>100 and len(b)>100; print('commands:', len(a), len(b))"

# 3. every registered {file:...} reference resolves
python3 - <<'PY'
import json, os
home = os.path.expanduser
broken = []
for p in (home('~/.config/opencode/opencode.json'), home('~/.opencode/opencode.json')):
    for name, cmd in json.load(open(p)).get('command', {}).items():
        tpl = cmd.get('template', '')
        if '{file:' in tpl:
            ref = tpl.split('{file:', 1)[1].split('}', 1)[0]
            if not os.path.exists(ref):
                broken.append((p, name))
assert not broken, broken
print('all file references resolve')
PY

# 4. opencode boots
opencode --version
```

## Evidence

Paste command output below when run.

- (pending)
