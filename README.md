# Prometheus Skill Pack

A comprehensive, enterprise-grade collection of Agent Skills for AI-assisted development. This package includes specialized skills for React entity management, Rust development, UI/UX design, DevOps workflows, and more.

## 🎯 Features

- **🔷 React Skills**: Entity management, component patterns, state management, and modern React best practices
- **🦀 Rust Skills**: Systems programming, memory safety, concurrency patterns, and Rust ecosystem tools
- **🎨 UI/UX Skills**: Design systems, accessibility, responsive design, and user experience patterns
- **⚙️ DevOps Skills**: CI/CD, containerization, infrastructure as code, and deployment workflows
- **✅ Testing Skills**: TDD, integration testing, E2E testing, and testing best practices
- **📚 Documentation Skills**: API documentation, technical writing, and code documentation patterns

## 📦 Installation

### Prerequisites

This repository uses git submodules for imported skills. When cloning, use:

```bash
# Clone with submodules
git clone --recurse-submodules https://github.com/gqadonis/prometheus-skill-pack.git

# Or if already cloned, initialize submodules
git submodule init
git submodule update
```

### Claude Code

#### Install from Marketplace

```bash
# Add the marketplace
/plugin marketplace add gqadonis/prometheus-skill-pack

# Install the full package
/plugin install prometheus-skill-pack

# Or install specific skill domains
/plugin install prometheus-react-skills
/plugin install prometheus-rust-skills
/plugin install prometheus-uiux-skills
```

#### Manual Installation

**Project-scoped** (recommended for teams):
```bash
git clone https://github.com/gqadonis/prometheus-skill-pack.git
cp -r prometheus-skill-pack/.claude-plugin/skills/* .claude/skills/
```

**User-scoped** (personal use):
```bash
git clone https://github.com/gqadonis/prometheus-skill-pack.git
cp -r prometheus-skill-pack/.claude-plugin/skills/* ~/.claude/skills/
```

### VS Code / GitHub Copilot

```bash
git clone https://github.com/gqadonis/prometheus-skill-pack.git
cp -r prometheus-skill-pack/skills/* .github/skills/
```

### Other Agents

This package follows the [agentskills.io](https://agentskills.io) standard and works with any compatible AI agent:
- Claude Code
- GitHub Copilot
- VS Code Copilot
- Cursor
- OpenAI Codex
- And more

## 🚀 Quick Start

After installation, skills are automatically available. Use them by:

1. **Natural invocation** - AI automatically loads relevant skills based on your task
2. **Explicit invocation** - Use `/skill-name` in Claude Code or mention the skill in your prompt
3. **Browse skills** - Run `/skills` to see all available skills

## 📖 Skill Categories

### Imported Skills
Located in `skills/imported/` (git submodules)

**artifact-refiner** (v1.1.0)
- PMPO-driven artifact refinement engine
- Supports logos, UI components, A2UI specs, images, content, and code
- Complete plugin with agents, hooks, and scripts
- Repository: [artifact-refiner-skill](https://github.com/GQAdonis/artifact-refiner-skill)

> **Note**: Imported skills are managed as git submodules. See [Submodule Management Guide](docs/SUBMODULES.md) for update procedures.

### React Skills
Located in `skills/react/`

- `react-entity-crud` - Complete CRUD operations for entity management
- `react-entity-forms` - Advanced form handling with validation
- `react-entity-tables` - Data tables with sorting, filtering, pagination
- `react-state-patterns` - Modern state management patterns
- `react-hooks-best-practices` - Custom hooks and composition patterns

### Rust Skills
Located in `skills/rust/`

- `rust-memory-safety` - Ownership, borrowing, and lifetimes
- `rust-concurrency` - Async/await, tokio, and concurrent patterns
- `rust-error-handling` - Result types and error propagation
- `rust-testing` - Unit tests, integration tests, and benchmarks
- `rust-cli-apps` - Command-line application development

### UI/UX Skills
Located in `skills/ui-ux/`

- `design-systems` - Component libraries and design tokens
- `accessibility-wcag` - WCAG compliance and a11y patterns
- `responsive-design` - Mobile-first and adaptive layouts
- `user-research` - User testing and feedback integration
- `prototyping` - Wireframing and interactive prototypes

### DevOps Skills
Located in `skills/devops/`

- `ci-cd-pipelines` - GitHub Actions, GitLab CI, and Jenkins
- `docker-containers` - Containerization and multi-stage builds
- `kubernetes-deployment` - K8s manifests and deployment strategies
- `infrastructure-code` - Terraform and infrastructure automation
- `monitoring-observability` - Logging, metrics, and tracing

### Testing Skills
Located in `skills/testing/`

- `test-driven-development` - TDD workflow and patterns
- `integration-testing` - API and service integration tests
- `e2e-testing` - End-to-end test automation
- `test-coverage` - Coverage analysis and improvement
- `performance-testing` - Load testing and benchmarking

## 🛠️ Development

### Project Structure

```
prometheus-skill-pack/
├── .claude-plugin/          # Claude Code plugin configuration
│   ├── plugin.json         # Plugin manifest
│   ├── skills/             # Symlinks to main skills directory
│   └── agents/             # Symlinks to main agents directory
├── skills/                 # Main skills directory (agentskills.io format)
│   ├── react/              # React-specific skills
│   ├── rust/               # Rust-specific skills
│   ├── ui-ux/              # UI/UX design skills
│   ├── devops/             # DevOps workflow skills
│   ├── testing/            # Testing methodology skills
│   └── documentation/      # Documentation skills
├── agents/                 # Specialized agents
├── hooks/                  # Automation hooks
├── shared/                 # Shared resources
│   ├── scripts/            # Reusable scripts
│   ├── templates/          # File templates
│   └── utils/              # Utility functions
├── marketplace/            # Marketplace configuration
├── docs/                   # Documentation
├── examples/               # Usage examples
└── scripts/                # Build and validation tools
```

### Git Submodules

This repository includes imported skills as git submodules. Key commands:

```bash
# Initialize submodules after cloning
git submodule init && git submodule update

# Update all imported skills to latest
git submodule update --remote

# Update specific skill
cd skills/imported/artifact-refiner
git pull origin main
cd ../../..
git add skills/imported/artifact-refiner
git commit -m "Update artifact-refiner"
```

See [docs/SUBMODULES.md](docs/SUBMODULES.md) for complete submodule management guide.

### Commands

```bash
# Validate all skills (including imported)
npm run validate

# Validate a specific skill
npm run validate:skill skills/react/react-entity-crud

# Build marketplace distribution
npm run build

# Run tests
npm test

# Lint skills
npm run lint

# Format code and markdown
npm run format

# Check formatting
npm run check-format

# Watch for changes during development
npm run dev

# Install to user scope
npm run install:user

# Install to project scope
npm run install:project
```

### Creating New Skills

1. Choose the appropriate category directory in `skills/`
2. Create a new skill directory with kebab-case naming
3. Add `SKILL.md` with required frontmatter:

```markdown
---
name: skill-name
description: Clear description of what this skill does and when to use it
license: MIT
---

# Skill Name

## Instructions

Step-by-step instructions for Claude...

## Examples

Concrete usage examples...
```

4. Add optional subdirectories:
   - `scripts/` - Executable code
   - `references/` - Detailed documentation
   - `assets/` - Templates, schemas, data files

5. Validate the skill: `npm run validate:skill skills/category/skill-name`

## 📋 AgentSkills.io Compliance

This package fully complies with the [agentskills.io](https://agentskills.io/specification) standard:

✅ Required `SKILL.md` with YAML frontmatter
✅ Standard directory structure (`scripts/`, `references/`, `assets/`)
✅ Portable across agent platforms
✅ Progressive disclosure for context management
✅ Self-contained, executable scripts
✅ Forward-slash paths for cross-platform compatibility

## 🔧 Shared Resources

### Scripts
Located in `shared/scripts/`, these are reusable utilities that can be referenced by any skill:
- Validation scripts
- Code generation templates
- Common transformations
- Deployment helpers

### Templates
Located in `shared/templates/`, these provide boilerplate for common patterns:
- Component scaffolds
- Configuration files
- Test templates
- Documentation templates

### Utils
Located in `shared/utils/`, these are helper functions and utilities:
- JSON/YAML parsers
- File system utilities
- String manipulation
- Common algorithms

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Follow the skill creation guidelines
4. Run validation: `npm run validate`
5. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) for details

## 🔗 Links

- [AgentSkills.io Specification](https://agentskills.io/specification)
- [Claude Code Documentation](https://code.claude.com/docs)
- [GitHub Repository](https://github.com/gqadonis/prometheus-skill-pack)
- [Issue Tracker](https://github.com/gqadonis/prometheus-skill-pack/issues)

## 🙏 Acknowledgments

Built with inspiration from:
- [Anthropic Skills Repository](https://github.com/anthropics/skills)
- [AgentSkills.io Community](https://agentskills.io)
- Claude Code ecosystem

---

**Made with ❤️ for the AI development community**
