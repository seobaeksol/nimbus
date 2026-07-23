# Nimbus

Nimbus is a Windows 11 file-workbench built with Rust and GPUI. It keeps familiar
Explorer interactions while making multi-location work, safe file operations,
temporary collections, previews, and folder context available in one window.

## Nimbus 1.0

- One to four independent, recursively split file panels with resizable boundaries
- Per-panel tabs, pinned tabs, history, sorting, filtering, selection, and scroll restore
- Single, two-column, two-row, three-panel, and 2×2 layout presets
- Named workspaces and automatic last-session restore
- A resizable tabbed Sidebar for Navigation, Shelf, Git, Folder Info, Statistics, and Search
- Multiple persistent or session-only Shelves with copy, move, ZIP, path-copy, and batch-rename actions
- Background directory loading, file watching, recursive search, Git reads, size calculation, and statistics
- Virtualized file lists for large folders
- Quick Look for images and text, with metadata fallbacks for PDF, video, Office, and unknown formats
- Copy, move, rename, batch rename, new folder, ZIP, and Recycle Bin operations
- Conflict preflight with overwrite, skip, or keep-both decisions
- A sequential operation queue with pause, resume, cancel, summaries, and safe undo
- Internal drag-and-drop between panels (`Shift` changes copy to move)
- Command Palette with context-aware enablement and discoverable shortcuts
- Windows, PowerShell, file URI, and WSL path formats
- Windows associated-app launching, File Explorer reveal, PowerShell launch, known folders, drives, UNC, and long paths

## Run

```powershell
cargo run
```

Open a named workspace directly:

```powershell
cargo run -- --workspace "Release review"
```

Nimbus saves state to `%APPDATA%\Nimbus\state.json` and keeps the prior state as
`state.bak` when replacing it. Overwritten file-operation targets are staged below
the adjacent `undo` directory until their operation is undone or the data is
manually cleaned up.

## Core shortcuts

| Shortcut | Action |
| --- | --- |
| `Ctrl+Shift+P` | Command Palette |
| `Space` | Quick Look |
| `Enter` | Open selected item |
| `F2` | Rename |
| `Delete` | Review move to Recycle Bin |
| `F5` | Refresh active panel |
| `Alt+Left/Right` | Back / forward |
| `Ctrl+L` | Focus address field |
| `Ctrl+F` | Search |
| `Ctrl+T` / `Ctrl+W` | New / close tab |
| `Ctrl+Shift+V/H` | Split into columns / rows |
| `Ctrl+C/X/V` | Copy / cut / paste through Nimbus |
| `Ctrl+Shift+S` | Add selection to Shelf |
| `Ctrl+Z` | Undo the latest safe operation |

Search accepts filter tokens alongside fuzzy name text:
`ext:pdf`, `glob:**/*.rs`, `min:10mb`, `max:1gb`, and `hidden:true`.

## Verify

```powershell
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

PDF, video, and Office previews intentionally use a metadata-first fallback in
1.0; opening the associated application remains available from Quick Look.
Git integration is read-only by design.
