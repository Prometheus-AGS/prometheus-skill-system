# Scheduled Jobs

Prometheus scheduled maintenance jobs. Each job has a macOS launchd plist and a Linux cron snippet.

## Jobs

| Script | Schedule | Purpose |
|--------|----------|---------|
| `mem0-compress.sh` | Weekly, Sunday 03:00 | Compress surreal-memory memories for `prometheus-skill-pack` scope |
| `pk-lint.sh` | Weekly, Saturday 03:00 | Run `pk lint --fix` on all skills |

## macOS installation (launchd)

For always-on local MCP services on macOS, prefer the repo-level service manager:

```bash
bash scripts/prometheus-services.sh install
bash scripts/prometheus-services.sh load
bash scripts/prometheus-services.sh status
```

It installs user LaunchAgents for the logged-in user and keeps `pk-cherry`
(`:8942`) and `forge mcp` (`:8943`) running. `surreal-memory-server` stays
Docker-managed on `:23001`.

The scheduled maintenance jobs below are separate periodic jobs.

```bash
# Substitute actual paths
REPO=$(git -C ~/.claude/skills rev-parse --show-toplevel 2>/dev/null || echo "$HOME/.claude/plugins/prometheus-skill-pack")
LOG_DIR="$HOME/.prometheus"
mkdir -p "$LOG_DIR"

# Edit the plist, replace PROMETHEUS_SKILL_PACK_ROOT and PROMETHEUS_LOG_DIR
sed -e "s|PROMETHEUS_SKILL_PACK_ROOT|${REPO}|g" \
    -e "s|PROMETHEUS_LOG_DIR|${LOG_DIR}|g" \
    ai.prometheus.mem0-compress.plist \
    > ~/Library/LaunchAgents/ai.prometheus.mem0-compress.plist

launchctl load ~/Library/LaunchAgents/ai.prometheus.mem0-compress.plist
```

## Linux installation (cron)

```bash
REPO=<path-to-prometheus-skill-pack>
(crontab -l 2>/dev/null; echo "0 3 * * 0 ${REPO}/shared/scripts/mem0-compress.sh >> ~/.prometheus/mem0-compress-cron.log 2>&1") | crontab -
(crontab -l 2>/dev/null; echo "0 3 * * 6 ${REPO}/shared/scripts/pk-lint.sh >> ~/.prometheus/pk-lint-cron.log 2>&1") | crontab -
```

## Manual run

```bash
bash shared/scripts/mem0-compress.sh
bash shared/scripts/pk-lint.sh
```
