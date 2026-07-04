# UI Architecture — Single-View vs Two-Zone Navigation

## Date
2026-07-04

## The Problem

When evolving a single-item UI (one research job) into a multi-item UI (many concurrent research jobs), the natural temptation is to create **two zones of navigation**: a "workspace" zone for global actions and a "job context" zone for per-job actions. The sidebar switches between these modes based on whether a job is selected.

I fell into this trap. My redesign used:
- `workspaceView`: dashboard | new | kbs | settings
- `jobView`: overview | report | sources | graph | contradictions | activity | export
- `activeJobId`: string | null
- A job switcher dropdown to switch between jobs
- The sidebar physically changed its nav items when entering/exiting job context

## The Better Approach (Fable 5)

Fable 5 used a **single `currentView` state** with horizontal tabs for detail views:
- `currentView`: dashboard | new | kb | components | settings | **job**
- `jobTab`: overview | sources | graph | contradictions | outputs | config
- `activeJobId`: string | null

Jobs are **not a navigation mode**. They are items in a list. Clicking a job is like clicking an email in a mail client or an issue in a tracker — it opens the detail view.

## The Pattern

This is how most modern multi-item apps work:

| App | Sidebar | Detail View |
|-----|---------|-------------|
| **Linear** | Fixed workspace items + dynamic issue lists | Issue tabs in main content |
| **GitHub** | Fixed org/repos + dynamic PR/issue lists | PR tabs in main content |
| **Slack** | Fixed workspace + dynamic channel/DM lists | Channel content in main area |
| **Gmail** | Fixed labels + dynamic conversation list | Email content in main area |
| **Fable 5 UI** | Fixed workspace + dynamic job status lists | Job tabs in main content |

The pattern is:
1. **Sidebar** = fixed workspace items + dynamic item lists grouped by status
2. **Main content** = dashboard (list/grid view) OR detail view with horizontal tabs
3. **No mode switching** — the sidebar never changes its fundamental structure

## Why Two-Zone Is Wrong

1. **Cognitive overload**: Users must understand two different navigation models
2. **Lost context**: When you switch from job view back to workspace, you lose the job context entirely
3. **Dropdown friction**: Job switcher requires extra clicks and is not discoverable
4. **Mobile complexity**: Two-zone models are harder to adapt to mobile bottom nav
5. **Template bloat**: Two separate nav templates, conditional rendering everywhere

## The Rule

> **Items are lists, not nav modes. Detail views are tabs, not sidebar items.**

When you have a collection of items (jobs, emails, issues, PRs):
- Show them in the sidebar as **lists grouped by status**
- Clicking an item opens a **detail view with horizontal tabs** in the main content
- The sidebar **never** changes its fundamental structure — it just shows/hides dynamic sections

## Related Learnings

- [CSS Grid + Fixed Position Trap](css-grid-fixed-position-trap.md) — a related layout lesson from the same redesign effort
- [Read Before Build](read-before-build.md) — the importance of reading the existing file before redesigning
