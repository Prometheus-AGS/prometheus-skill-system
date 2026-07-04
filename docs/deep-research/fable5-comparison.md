# Fable 5 UI Analysis — What It Did Better

**Date:** 2026-07-04  
**Context:** Comparing Anthropic Fable 5's multi-tenancy redesign of the `tj-deep-research.html` UI against my own redesign attempt. Both started from the same original file.

---

## Executive Summary

Fable 5 produced a **significantly better UI architecture** by using a simpler, more conventional navigation pattern. Where I over-engineered a two-zone navigation model with mode-switching sidebars and job switcher dropdowns, Fable 5 used a **single-view state + dynamic sidebar sections + horizontal tabs** — the same pattern used by Linear, GitHub, and Slack. The result is more intuitive, more maintainable, and more mobile-friendly.

---

## Detailed Comparison

### 1. State Model

| Dimension | My Approach | Fable 5 Approach | Winner |
|-----------|------------|------------------|--------|
| Top-level state | `workspaceView` + `jobView` (two variables) | `currentView` (one variable) | **Fable 5** |
| Job sub-state | `jobView` (7 values) | `jobTab` (6 values) | **Fable 5** |
| Job identity | `activeJobId` | `activeJobId` | Tie |
| State complexity | O(n×m) combinations | O(n) + O(m) linear | **Fable 5** |
| Mental model | "The app has two modes" | "I click an item to see details" | **Fable 5** |

**My mistake:** I created two separate navigation domains. This forces the user to understand that the app "changes hats" when they enter/exit a job context. Fable 5's single `currentView` with `job` as just one of the views is much simpler — entering a job is like opening an email, not switching operating modes.

---

### 2. Navigation Architecture

| Dimension | My Approach | Fable 5 Approach | Winner |
|-----------|------------|------------------|--------|
| Sidebar behavior | **Switches entirely** between workspace nav and job context nav | **Always shows** workspace items + dynamic job lists | **Fable 5** |
| Job switching | Dropdown at top of sidebar | Click job in sidebar list | **Fable 5** |
| Job tabs location | Sidebar nav items | Horizontal tabs in main content | **Fable 5** |
| Dashboard | Placeholder / settings redirect | Real view with stats + job cards | **Fable 5** |
| New Research | Workspace-level wizard | Always accessible via sidebar button | **Fable 5** |

**Why Fable 5 wins:** The sidebar is the user's anchor. Changing its entire contents is disorienting. Fable 5 keeps the sidebar stable — workspace items are always there, and jobs appear as dynamic sections below. This is exactly how Linear (sidebar shows teams + projects + issues) and GitHub (sidebar shows orgs + repos + PRs) work.

---

### 3. Sidebar Design

| Dimension | My Approach | Fable 5 Approach | Winner |
|-----------|------------|------------------|--------|
| Workspace items | Dashboard, New Research, KBs, Settings | Dashboard, KBs, Components, Settings | **Fable 5** |
| Job lists | Hidden in dropdown | Visible as dynamic sections (Running, Queued, Completed) | **Fable 5** |
| New Research button | In workspace nav | Prominent button at top of sidebar | **Fable 5** |
| Status indicators | In dropdown only | In sidebar with colored dots + pulse animation | **Fable 5** |
| Progress visibility | Requires opening job | Visible in sidebar as percentage | **Fable 5** |

**Fable 5's sidebar structure:**
```
┌─ [New Research] button
├─ Workspace
│  ├─ Dashboard
│  ├─ Knowledge Bases
│  ├─ Components
│  └─ Settings
├─ Running (dynamic)
│  ├─ ● Permian Basin comps — 62%
│  └─ ● ...
├─ Queued (dynamic)
│  └─ ○ AP invoice automation
└─ Completed (dynamic)
   └─ ✓ Multi-agent architectures
```

My sidebar structure:
```
┌─ [Job Switcher dropdown] ← extra click, hides jobs
├─ Workspace
│  ├─ Dashboard
│  ├─ New Research
│  ├─ KBs
│  └─ Settings
│     ← OR when job active →
├─ Job Context
│  ├─ Overview
│  ├─ Report
│  ├─ Sources
│  ├─ Graph
│  ├─ Contradictions
│  ├─ Activity
│  └─ Export
```

The problem: when you switch to a job, the entire sidebar changes. You lose your workspace context. Fable 5 never does this.

---

### 4. Dashboard

| Dimension | My Approach | Fable 5 Approach | Winner |
|-----------|------------|------------------|--------|
| Exists? | Minimal / redirect | Yes — full feature | **Fable 5** |
| Stats | Not shown | Stat cards (Running, Queued, Completed, Sources, Contradictions) | **Fable 5** |
| Job cards | Not shown | Grid of job cards with progress bars, status pills, metadata | **Fable 5** |
| New job CTA | Separate nav item | "New Research Job" card in grid | **Fable 5** |
| Mobile | N/A | Stats stack, cards become full-width | **Fable 5** |

A dashboard is the natural "home" view for a multi-item application. Fable 5 understood this; I treated it as an afterthought.

---

### 5. Job Detail View (Tabs)

| Dimension | My Approach | Fable 5 Approach | Winner |
|-----------|------------|------------------|--------|
| Tab style | Vertical sidebar nav items | Horizontal tab strip in content area | **Fable 5** |
| Tab count | 7 tabs | 6 tabs | Tie |
| Tab labels | Overview, Report, Sources, Graph, Contradictions, Activity, Export | Overview, Sources, Graph, Contradictions, Outputs, Config | **Fable 5** |
| Report location | Separate tab | Inside "Outputs" tab with export actions | **Fable 5** |
| Config location | Settings panel | "Run Config" tab in job context | **Fable 5** |

**Fable 5's tab grouping is smarter:**
- **Overview** = progress, A2UI media card, AG-UI trace, threads
- **Sources** = source list with search + filter
- **Graph** = knowledge graph visualization
- **Contradictions** = contradiction list
- **Outputs** = report + export actions (grouped naturally)
- **Config** = immutable launch snapshot + clone button

My "Report" and "Export" as separate tabs felt disconnected. Fable 5 grouping them under "Outputs" makes sense — they are both things you get out of the job. "Config" being inside the job (not in global settings) is also correct — it's the snapshot of how this specific job was launched.

---

### 6. Mobile UX

| Dimension | My Approach | Fable 5 Approach | Winner |
|-----------|------------|------------------|--------|
| Bottom nav items | Not well defined | Research, Jobs, Active, Settings, More | **Fable 5** |
| "Active" button | N/A | Opens last active job — brilliant shortcut | **Fable 5** |
| Sidebar | Drawer from bottom | Same drawer pattern, but with handle | **Fable 5** |
| Pull-to-refresh | Not implemented | Implemented with spinner indicator | **Fable 5** |
| Touch gestures | Basic swipe detection | Swipe to open/close sidebar + job switching | **Fable 5** |

The "Active" button in the bottom nav is a particularly nice touch — it remembers which job you were working on and jumps back to it.

---

### 7. Data Model & Code Organization

| Dimension | My Approach | Fable 5 Approach | Winner |
|-----------|------------|------------------|--------|
| Computed properties | Some | Extensive getters: `runningJobs`, `queuedJobs`, `completedJobs`, `activeJob`, `aj`, `pageTitle`, `statusText`, `statusPillClass`, `currentStageName`, `filteredSources` | **Fable 5** |
| Template logic | Heavy conditional rendering | Clean — relies on computed properties | **Fable 5** |
| Demo data | 3 jobs with basic data | 3 jobs with rich data, ambient simulation, live traces | **Fable 5** |
| Ambient activity | Not implemented | `startAmbient()` simulates live AG-UI traces every 7s | **Fable 5** |
| Source detail | Inline list only | Modal with full metadata, author, word count, URL | **Fable 5** |
| History panel | Not implemented | Slide-out panel with recent jobs, clickable | **Fable 5** |

Fable 5's use of computed getters is exemplary. The `aj` (active job) getter that returns a safe default object when no job is active prevents null-checking everywhere. The `pageTitle` and `statusText` getters adapt dynamically based on current view. This is clean, reactive Alpine.js patterns done right.

---

### 8. Settings Panel

| Dimension | My Approach | Fable 5 Approach | Winner |
|-----------|------------|------------------|--------|
| Sections | 10 (5 real + 5 stubbed) | 9 (all functional) | **Fable 5** |
| Navigation | Left rail in settings panel | Same left rail pattern | Tie |
| Field layout | Two-column (label + control) | Same pattern, but better responsive handling | Tie |
| Responsiveness | Basic | Collapses to horizontal scroll rail at 860px | **Fable 5** |
| Integration endpoints | Not shown | AG-UI stream, MCP endpoint, A2A agent card with code blocks | **Fable 5** |

Fable 5's "Integrations" section with protocol endpoints and code blocks is a particularly nice touch — it shows the system architecture directly in the UI.

---

### 9. What I Did Better (or Equally Well)

Despite Fable 5's overall superiority, there are a few areas where my approach was comparable or better:

| Aspect | My Approach | Fable 5 | Assessment |
|--------|------------|---------|------------|
| CSS Grid trap documentation | Documented in comments | Also documented in comments | Tie — both learned from the same mistake |
| PWA readiness | Full meta tags, manifest, icons | Same | Tie |
| FOUC guard | Pre-apply theme before paint | Same | Tie |
| Keyboard shortcuts | `/` for search, `Escape` to close, `n` for new | Same | Tie |
| Accessibility | `min-height: 44px` touch targets, `aria-label` | Same | Tie |
| Theme toggle | In sidebar footer | Same | Tie |
| HLS audio playback | Implemented | Implemented | Tie |

Most of these are tied because they were inherited from the original file that both Fable 5 and I started from.

---

## Key Architectural Principles from Fable 5

### 1. Single-View State Pattern

```javascript
// Fable 5: Simple, linear
state: {
  currentView: 'dashboard' | 'new' | 'kb' | 'components' | 'settings' | 'job',
  jobTab: 'overview' | 'sources' | 'graph' | 'contradictions' | 'outputs' | 'config',
  activeJobId: string | null
}

// My approach: Two separate navigation domains
state: {
  workspaceView: 'dashboard' | 'new' | 'kbs' | 'settings',
  jobView: 'overview' | 'report' | 'sources' | 'graph' | 'contradictions' | 'activity' | 'export',
  activeJobId: string | null
}
```

The single-view approach eliminates the need for a "mode switch" in the UI. The sidebar never changes its fundamental structure.

### 2. Items Are Lists, Not Nav Modes

Jobs are data items. They should appear in lists in the sidebar, not as navigation modes. When you click a job, you navigate to the `job` view with that job's ID. The sidebar stays the same.

### 3. Horizontal Tabs for Detail Views

Detail views (the contents of a job) use horizontal tabs in the main content area. This is conventional and frees up sidebar space for item lists.

### 4. Dashboard Is Home

A multi-item app needs a real dashboard. It should show:
- Summary stats (counts by status)
- Item grid/cards with key metadata
- Clear CTA to create new items

### 5. Computed Properties Over Template Logic

Use getters for anything derived from state. Templates should be declarative, not computational.

---

## Actionable Recommendations

1. **Adopt the single-view state model** — Replace `workspaceView` + `jobView` with `currentView` + `jobTab`
2. **Restructure sidebar** — Always show Workspace (Dashboard, KBs, Components, Settings) + dynamic job lists below
3. **Add a real Dashboard** — Stat cards + job grid with "New Research Job" CTA card
4. **Move job tabs to horizontal** — Overview, Sources, Graph, Contradictions, Outputs, Config in content area
5. **Group Report + Export** — Under an "Outputs" tab instead of separate tabs
6. **Add Run Config tab** — Immutable launch snapshot per job with "Clone" button
7. **Add Source Detail Modal** — Clicking a source opens a modal with full metadata
8. **Add History Panel** — Slide-out recent jobs panel accessible from sidebar footer
9. **Add ambient simulation** — Live AG-UI traces for demo jobs to make the UI feel alive
10. **Add mobile bottom nav** — Research, Jobs, Active, Settings, More with "Active" shortcut
11. **Add pull-to-refresh** — Mobile-native refresh gesture
12. **Use computed getters more** — `activeJob`, `pageTitle`, `statusText`, `filteredSources`, etc.

---

## The Core Lesson

> **When evolving a single-item UI to multi-item, resist the temptation to create "zones" or "modes". Items are lists. Detail views are tabs. The sidebar is a stable anchor that shows both workspace navigation and dynamic item lists. This is how every great multi-item app works — from Gmail to Linear to GitHub.**

Fable 5 understood this instinctively. I over-engineered it. The pattern is: **single view state, dynamic sidebar sections, horizontal detail tabs, dashboard as home.**
