# Design Guidelines — crabase

## Philosophy
Minimal, airy, polished. Think Linear / Notion / Raycast / Vercel level of refinement. No visual clutter. Every pixel earns its place. DaisyUI is used sparingly — prefer raw Tailwind for tables, tabs, and data-dense components.

The app supports **two themes**: a light theme (default) and a dark theme. Both palettes are defined below. The user toggles between them in Settings (Cmd+Shift+P → "Settings").

Implementation: use Tailwind's `dark:` modifier with the `class` strategy. Apply `dark` class to `<html>` based on the user's preference (stored in settings).

## Light Theme — Color Palette

| Role | Hex | Tailwind |
|---|---|---|
| Background | `#FFFFFF` | `bg-white` |
| Surface/Panel | `#F9FAFB` | `bg-gray-50` |
| Border | `#E5E7EB` | `border-gray-200` |
| Border subtle | `#F3F4F6` | `border-gray-100` |
| Text primary | `#111827` | `text-gray-900` |
| Text secondary | `#6B7280` | `text-gray-500` |
| Text muted | `#9CA3AF` | `text-gray-400` |
| Accent/primary | `#6366F1` | `text-indigo-500` / `bg-indigo-500` |
| Accent hover | `#4F46E5` | `hover:bg-indigo-600` |
| Selection | `#EEF2FF` | `bg-indigo-50` |
| Row added | `#ECFDF5` bg + `border-l-2 border-emerald-500` | `bg-emerald-50` |
| Row modified | `#FFFBEB` bg + `border-l-2 border-amber-500` | `bg-amber-50` |
| Row deleted | `#FEF2F2` bg + `border-l-2 border-red-500` + `line-through opacity-60` | `bg-red-50` |
| Modified cell | `bg-amber-100/50` | |
| NULL value | `#D1D5DB` italic | `text-gray-300 italic` |

## Dark Theme — Color Palette

| Role | Hex | Tailwind |
|---|---|---|
| Background (deepest) | `#0A0A0A` | `dark:bg-neutral-950` |
| Surface (panels/sidebar) | `#111113` | `dark:bg-[#111113]` |
| Surface elevated (modals/dropdowns) | `#18181B` | `dark:bg-zinc-900` |
| Border (regular) | `#27272A` | `dark:border-zinc-800` |
| Border (subtle) | `#1F1F23` | `dark:border-[#1F1F23]` |
| Border (strong/focus) | `#3F3F46` | `dark:border-zinc-700` |
| Text primary | `#FAFAFA` | `dark:text-neutral-50` |
| Text secondary | `#A1A1AA` | `dark:text-zinc-400` |
| Text muted | `#71717A` | `dark:text-zinc-500` |
| Text disabled | `#52525B` | `dark:text-zinc-600` |
| Accent / primary | `#818CF8` | `dark:text-indigo-400` / `dark:bg-indigo-500` |
| Accent hover | `#A5B4FC` | `dark:hover:bg-indigo-400` |
| Selection highlight | indigo-500/25 | `dark:bg-indigo-500/25` |
| Row added (bg) | emerald-950/60 | `dark:bg-emerald-950/60` |
| Row added (border) | emerald-400 | `dark:border-emerald-400` |
| Row modified (bg) | amber-950/60 | `dark:bg-amber-950/60` |
| Row modified (border) | amber-400 | `dark:border-amber-400` |
| Row deleted (bg) | red-950/60 | `dark:bg-red-950/60` |
| Row deleted (border) | red-400 | `dark:border-red-400` |
| Modified cell | amber-900/40 + ring | `dark:bg-amber-900/40 dark:ring-1 dark:ring-amber-500/40` |
| NULL value | zinc-600 italic | `dark:text-zinc-600 italic` |
| Table header bg | `#0F0F11` | `dark:bg-[#0F0F11]` |
| Table row hover | white/3% | `dark:hover:bg-white/[0.03]` |
| Code/SQL bg | `#0D0D0F` | `dark:bg-[#0D0D0F]` |
| Code/SQL text | zinc-200 | `dark:text-zinc-200` |

### Dark Theme Tweaks
- **Shadows**: cap at `dark:shadow-black/40`. Prefer 1px top inner highlight `dark:ring-1 dark:ring-white/[0.06]` on elevated surfaces (Vercel trick).
- **Borders on overlays**: `dark:border-white/[0.08]` instead of solid zinc for softer seams.
- **Scrollbars**: `dark:scrollbar-thumb-zinc-800 dark:scrollbar-track-transparent`.
- **Focus rings**: `dark:ring-2 dark:ring-indigo-500/60 ring-offset-0`.
- **Dividers inside cards**: `dark:divide-white/[0.06]`.
- **Status row tints**: cap at 60% opacity, use 400-level foreground text for WCAG AA contrast.
- **Accent shift**: use `indigo-400` instead of `indigo-500` for text on dark backgrounds.

### Theme Toggle Implementation
```rust
// Add `dark` class to <html> conditionally
// Read theme preference from settings (light/dark/system)
// On toggle, persist to ~/.config/crabase/settings.json (or app_data_dir)
```

All component styles in this doc must use both light AND dark variants:
```html
<div class="bg-white dark:bg-neutral-950 text-gray-900 dark:text-neutral-50">
```

### Tauri Window Background (CRITICAL — currently broken)
The Tauri window itself has a white background that bleeds through in dark mode. This must be fixed:
- Set the window `backgroundColor` in `tauri.conf.json` to match the dark theme background (`#0A0A0A`)
- On startup, the backend reads `settings.json` and decides which background color to apply (light = `#FFFFFF`, dark = `#0A0A0A`)
- Light/dark switch at runtime requires window reload OR setting both `<html>` and Tauri window background
- The HTML/body must also have `bg-white dark:bg-neutral-950` set so there's no flash

### Table Text Contrast (currently broken)
In dark mode, the table cell text is currently rendered too dark to be readable. **All table cells must explicitly include `dark:text-zinc-200` (or `dark:text-neutral-50` for headings)**. Audit every table component to ensure no `text-gray-900` is missing its `dark:` variant.

### CodeMirror Theme Customization (currently broken)
The CodeMirror editor in dark mode currently uses `one-dark` which doesn't match the app palette. **Build a custom CodeMirror theme** that uses:
- Editor background: `#0A0A0A` (exact match with `dark:bg-neutral-950`)
- Active line highlight: `bg-white/[0.03]` (subtle, only for the focused line)
- Gutter background: `#0A0A0A` (no contrast with editor)
- Gutter text: `#52525B` (text-zinc-600)
- Selection: `#6366F1`/25% (`bg-indigo-500/25`)
- Cursor: `#FAFAFA` (text-neutral-50)
- Syntax token colors: keep CodeMirror defaults but ensure they all have ≥4.5:1 contrast on `#0A0A0A`
- The light theme variant uses `#FFFFFF` background with the existing light palette tokens

## Typography

- **UI font**: `Inter`, fallback `ui-sans-serif, system-ui, sans-serif`
- **Mono font**: `JetBrains Mono`, fallback `Fira Code, monospace` — used in SQL editor, table cells, code

| Element | Tailwind |
|---|---|
| Page title | `text-base font-semibold` |
| Section heading | `text-[13px] font-semibold` |
| Body / label | `text-[13px] font-normal` |
| Table header | `text-[11px] font-medium uppercase tracking-wider text-gray-500` |
| Table cell | `text-xs font-mono` |
| Code / SQL | `text-[13px] font-mono` |
| Badge / tag | `text-[11px] font-medium` |

## Spacing

Base unit: 4px (Tailwind scale). 

- Panels: `p-4`
- Table cells: `px-3 py-1.5`
- Section gaps: `gap-3` or `gap-4`
- Sidebar: `w-56` (224px)
- Tab bar height: `h-10`
- Toolbar height: `h-10`

## Components

### Sidebar (Tables list)
```
Container: bg-gray-50 border-r border-gray-200 w-56
Item:      px-3 py-1 text-[13px] rounded-md hover:bg-gray-100 cursor-pointer
Active:    bg-indigo-50 text-indigo-600
```

### Tabs
```
Container: flex items-center h-10 border-b border-gray-200 bg-white px-2 gap-0.5
Tab:       px-3 py-1.5 text-[13px] text-gray-500 rounded-t-md hover:text-gray-900 hover:bg-gray-50
Active:    text-gray-900 bg-white border-b-2 border-indigo-500
Close btn: opacity-0 group-hover:opacity-100 (appears on hover)
```

### Buttons
```
Primary:   bg-indigo-500 hover:bg-indigo-600 text-white text-[13px] font-medium px-3 py-1.5 rounded-md
Secondary: bg-white border border-gray-200 text-gray-700 hover:bg-gray-50 text-[13px] px-3 py-1.5 rounded-md
Ghost:     text-gray-500 hover:bg-gray-100 hover:text-gray-900 px-2 py-1 rounded-md
Danger:    bg-red-50 text-red-600 hover:bg-red-100 text-[13px] px-3 py-1.5 rounded-md
All:       transition-colors duration-100
```

### Inputs
```
bg-white border border-gray-200 rounded-md px-3 py-1.5 text-[13px]
focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500
```

### Modals / Dialogs
```
Overlay:   bg-black/40 backdrop-blur-sm
Panel:     bg-white rounded-lg shadow-xl border border-gray-200 max-w-lg
Header:    px-4 py-3 border-b border-gray-200
Body:      px-4 py-4
```

### Command Palette
```
Overlay:   backdrop-blur-sm bg-black/30
Panel:     bg-white rounded-xl shadow-2xl border border-gray-200 w-[560px] max-h-[400px] overflow-hidden mt-[20vh]
Input:     text-base px-4 py-3 border-b border-gray-100 w-full focus:outline-none
Group:     text-[11px] font-medium text-gray-400 uppercase px-4 py-2
Item:      px-4 py-2 flex items-center gap-3 text-[13px] cursor-pointer
Active:    bg-indigo-50 text-indigo-600
Shortcut:  text-[11px] text-gray-400 font-mono (right-aligned)
```

Navigate with arrow keys, select with Enter, dismiss with Escape.

### Table Finder (Cmd+P)
Same visual style as command palette. Shows table names, fuzzy-filtered. Selecting opens a new tab for that table.

## Data Table

### Structure
```
Table:       w-full text-xs font-mono
Header row:  bg-gray-50 border-b border-gray-200 sticky top-0 z-10
Header cell: px-3 py-2 text-left text-[11px] font-medium uppercase tracking-wider text-gray-500
             border-r border-gray-100 select-none
Body cell:   px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px]
```

### Row states
- **Hover**: `hover:bg-gray-50`
- **Selected**: `bg-indigo-50`
- **Added**: `bg-emerald-50 border-l-2 border-emerald-500`
- **Modified**: `bg-amber-50 border-l-2 border-amber-500`
- **Deleted**: `bg-red-50 border-l-2 border-red-500 line-through opacity-60`

### Inline editing
- Click cell to enter edit mode
- Cell gets `ring-2 ring-indigo-500/30 bg-white`, content auto-selected
- Escape reverts, Tab moves to next cell
- **Specialized editors by column type:**
  - `boolean` → checkbox
  - `enum` → select dropdown
  - `json` / `jsonb` → modal with syntax-highlighted editor
  - `text` / `varchar` → text input
  - `integer` / `numeric` → number input
  - `timestamp` / `date` → date/time input
- Modified cells: `bg-amber-100/50`

### Dirty state bar
When there are unsaved changes (edits, added rows, deleted rows), show a floating bar at the bottom:
```
fixed bottom-4 left-1/2 -translate-x-1/2
bg-white border border-gray-200 shadow-lg rounded-lg px-4 py-2
flex items-center gap-3 text-[13px]
```
Contains: summary text ("3 changes pending") + "Discard" (ghost button) + "Save changes" (primary button).

### Pagination
```
flex items-center justify-between px-3 py-2 border-t border-gray-200 bg-gray-50 text-[12px] text-gray-500
```
Show: page X of Y, rows per page selector, prev/next buttons.

## SQL Editor

- Toolbar: `h-10 flex items-center px-3 gap-2 border-b border-gray-200 bg-white`
- Run button: `bg-emerald-500 hover:bg-emerald-600 text-white` with play icon
- Editor: `bg-white font-mono text-[13px] leading-relaxed`
- Line gutter: `bg-gray-50 text-gray-400 text-right pr-2 select-none border-r border-gray-100`
- Results pane: below editor, resizable split. Same data table style but read-only.
- Error console: `bg-gray-900 text-red-400 font-mono text-xs p-3 overflow-y-auto`
- Comment toggle: `Cmd+/` on selected text prepends `-- `

## Animations

- Interactive elements: `transition-colors duration-100`
- Modals / command palette: `transition-all duration-150 ease-out` (scale 95%→100% + opacity)
- Sidebar collapse: `transition-[width] duration-200`
- Tab close: `transition-opacity duration-100`
- **No animations on data table rows/cells** (performance)

## Icons

Use **Lucide** icon set.
- Inline: `w-4 h-4`
- Toolbar: `w-5 h-5`
- Default color: `text-gray-400`, inherits on hover
- Always pair icons with text labels in buttons (except icon-only toolbar buttons)
- Keep icon usage minimal

## Saved Connections

Connection screen shows a list of saved connections (if any) above the connection string input. Each saved connection is a card:
```
border border-gray-200 rounded-lg px-4 py-3 hover:bg-gray-50 cursor-pointer
flex items-center justify-between
```
Name in `text-[13px] font-medium`, host/db in `text-[11px] text-gray-400`. Delete icon on hover.

## DaisyUI Config

- Theme: `light` base, override CSS variables with the palette above
- **Use DaisyUI for**: dropdowns, tooltips, toasts
- **Use raw Tailwind for**: tables, tabs, sidebar, command palette, modals
- Avoid DaisyUI's opinionated component styles on data-dense UI
