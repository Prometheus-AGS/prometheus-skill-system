# Orchestrator Lessons: Read Before Build

## The Pattern

When a user provides an existing file (HTML, CSS, code, config) and asks for improvements, **read it first** before building anything new.

### The Mistake

User says: "Make this HTML file mobile-responsive and PWA-ready."

User's existing file: `tj-deep-research.html` (129 KB, 3,342 lines) with correct layout, brand identity, meta tags, media player support, and working Alpine.js data model.

**What I did:** Didn't read the file. Instead, tried to rebuild the entire HTML from scratch based on branding docs and a skill template. Result: a fundamentally different design with a broken layout (grid + fixed position sidebar trap), missing features (hls.js, Open Graph tags, TJ branding), and a blank page (invalid Alpine.js CDN).

**What I should have done:** Read the file first, understand the existing structure, then add PWA meta tags, mobile drawer logic, and responsive breakpoints to the existing design.

## Why This Happens

1. **Skill-driven overconfidence** — The `branded-html-artifact` skill provides a workflow, but it's a template, not a source of truth. The user's existing file is always the source of truth.

2. **Assumption that "new features" = "new file"** — Adding PWA support or mobile responsiveness doesn't require rewriting a 3,000-line HTML file. It requires targeted additions to the existing file.

3. **Not checking for attachments** — The user had already attached the file in a previous turn. I should have remembered it was available.

## The Rule

**"Reference files are the source of truth. Skills are the tool, not the blueprint."**

When a user asks to modify or improve an existing artifact:

1. **Read** the existing file immediately
2. **Identify** what's already working vs. what needs changing
3. **Add** the requested features incrementally
4. **Preserve** the existing design system, data model, and interaction patterns
5. **Only rebuild** if the user explicitly says "start over" or "rewrite from scratch"

## Related

- [HTMX Layout: Grid + Fixed Position Trap](/htmx-layout-grid-fixed-position-trap.md)
- [Prometheus Deep Research Skill — Master Specification](/prometheus-deep-research-skill-master-spec.md)
