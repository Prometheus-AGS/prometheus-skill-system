# Cedar Policies

This directory contains [Cedar](https://www.cedarpolicy.com/) policy files that govern
the Prometheus self-learning pipeline's write operations against skill artifacts.

## Files

| File | Purpose |
|------|---------|
| `skill-mutation.cedar` | Policies for skill.mutate, skill.generate, skill.promote, trace.capture |
| `entities.json` | Entity hierarchy: agent groups, skill domains, named agents |

## How It Works

Cedar is **default-deny**: if no `permit` policy matches a request, it is denied.
`forbid` policies override any matching `permit`.

The `prometheus-cedar` crate loads these files and evaluates authorization requests
at the Skill Mutation PEP (Policy Enforcement Point).

## Environments

| Environment | Behavior |
|-------------|----------|
| `development` | All operations permitted |
| `staging` | Mutations require `validation_passed`; promotions require `human_approved` + `test_pass_rate >= 0.95` |
| `production` | All mutations forbidden by default |

## Regulated Verticals

Additional `forbid` policies for healthcare (require `audit_trail_id`) and
financial (require `dual_approval`) contexts.

## Customization

1. Edit `skill-mutation.cedar` to add/modify policies
2. Edit `entities.json` to add agents or groups
3. Set `$PROMETHEUS_POLICY_DIR` to override the policy directory
4. Run `prometheus policy validate` to check syntax
