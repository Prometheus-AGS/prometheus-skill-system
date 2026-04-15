# Shared Scripts

This directory contains reusable scripts that can be referenced by multiple skills across the package.

## Purpose

Shared scripts promote code reuse and consistency. Instead of duplicating utility functions across multiple skills, common functionality lives here.

## Structure

```
shared/scripts/
├── validators/          # Validation utilities
├── generators/          # Code generation scripts
├── formatters/          # Formatting utilities
├── parsers/            # File parsing utilities
└── common/             # General-purpose utilities
```

## Usage in Skills

Reference shared scripts from your SKILL.md:

```markdown
## Instructions

1. Validate the configuration:
   ```bash
   bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/validators/validate-config.sh config.json
   ```

2. Generate boilerplate:
   ```bash
   python3 ${CLAUDE_PLUGIN_ROOT}/shared/scripts/generators/generate-component.py --name MyComponent
   ```
```

## Guidelines

### Self-Contained Scripts

All scripts should be self-contained with minimal dependencies:

✅ **Good** - Uses package runners:
```python
# /// script
# dependencies = [
#   "pyyaml",
# ]
# ///

import yaml
# Script logic...
```

✅ **Good** - Single-file with no deps:
```bash
#!/bin/bash
# Pure bash script with no external dependencies
```

❌ **Bad** - Assumes global packages:
```python
# Requires: pip install pyyaml
import yaml  # Might not be installed!
```

### Structured Output

Scripts should output structured data (JSON) when possible:

```bash
#!/bin/bash
# Good: JSON output
echo '{"status": "success", "result": "value"}'

# Avoid: Unstructured output
echo "The operation succeeded with result: value"
```

### Error Handling

Always handle errors gracefully:

```python
import sys
import json

try:
    # Script logic
    result = {"status": "success", "data": "..."}
except Exception as e:
    result = {"status": "error", "message": str(e)}
    sys.exit(1)
finally:
    print(json.dumps(result))
```

### Documentation

Each script should include:

1. **Header comment** describing purpose:
   ```bash
   #!/bin/bash
   # validate-config.sh
   # Validates JSON/YAML configuration files
   # Usage: validate-config.sh <file>
   ```

2. **Help flag**:
   ```bash
   if [[ "$1" == "--help" ]]; then
       echo "Usage: $0 <config-file>"
       echo "Validates configuration file format"
       exit 0
   fi
   ```

3. **Input validation**:
   ```bash
   if [[ $# -lt 1 ]]; then
       echo "Error: Config file required" >&2
       exit 1
   fi
   ```

## Categories

### Validators

Scripts that check file formats, configurations, or data validity:
- `validate-json.sh` - JSON syntax validation
- `validate-yaml.sh` - YAML syntax validation
- `validate-env.sh` - Environment variable validation

### Generators

Scripts that create boilerplate code or files:
- `generate-component.py` - React component scaffolding
- `generate-test.py` - Test file generation
- `generate-docs.sh` - Documentation generation

### Formatters

Scripts that format or transform data:
- `format-json.sh` - Pretty-print JSON
- `format-yaml.sh` - Pretty-print YAML
- `convert-json-yaml.py` - Format conversion

### Parsers

Scripts that extract information from files:
- `parse-frontmatter.py` - Extract YAML frontmatter
- `parse-package-json.js` - Extract package.json fields
- `parse-cargo-toml.py` - Extract Cargo.toml fields

### Common

General-purpose utilities:
- `file-utils.sh` - File system operations
- `string-utils.sh` - String manipulation
- `network-utils.sh` - Network operations

## Contributing

When adding a shared script:

1. Place it in the appropriate category directory
2. Make it executable: `chmod +x script-name.sh`
3. Include documentation header
4. Add entry to this README
5. Test with multiple skills
6. Update skills that can benefit from it

## Best Practices

1. **Platform compatibility**: Use forward slashes, avoid platform-specific commands
2. **Minimal dependencies**: Prefer built-in tools over external packages
3. **Package runners**: Use `npx`, `uvx`, `bunx` for external tools
4. **JSON output**: Structured data for easy parsing
5. **Error codes**: Non-zero exit on failure
6. **Documentation**: Clear usage instructions
7. **Testing**: Validate scripts work across different environments

## Environment Variables

Scripts can access these Claude Code variables:
- `$CLAUDE_PLUGIN_ROOT` - Root of the plugin directory
- `$REPO_ROOT` - Root of the current repository
- `$HOME` - User home directory

## Security

⚠️ **Important**:
- Never include API keys or secrets
- Validate all input parameters
- Sanitize file paths to prevent injection
- Use `set -euo pipefail` in bash scripts
- Don't execute untrusted input

## Examples

See `examples/` directory for:
- Complete working scripts
- Integration patterns
- Testing approaches
