# Nimbus 1.0 Acceptance Notes

This document maps the PRD's initial success criteria to the implemented 1.0
behavior.

| PRD success criterion | Nimbus 1.0 implementation |
| --- | --- |
| Open three folders without window switching | Three-panel preset plus recursively resizable panel model |
| Move files between those folders | Active/target panel labels, transfer buttons, and panel drop targets with preflight |
| Collect files from multiple locations | Named, persistent or session-only Shelves |
| Copy a Shelf into one destination | Shelf Copy/Move actions feed the same conflict-aware operation queue |
| Preview without opening an associated app | `Space` Quick Look with image and text providers and metadata fallback |
| Keep navigation responsive | Directory, Git, search, preview, size, statistics, and operations run off the UI thread |
| Review bulk conflicts before execution | Operation review lists every source/destination and applies keep-both, skip, or overwrite |
| Restore the last working context | Atomic JSON state for layouts, ratios, tabs, paths, sorting, filtering, scroll positions, Sidebar, Shelves, and named workspaces |
| Complete basic work without documentation | Familiar double-click/Enter, `Ctrl+C/X/V`, `F2`, `Delete`, navigation controls, tooltips, empty states, and Command Palette |

## Verification gates

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo test --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- Windows launch smoke test
- Single-panel, two-panel, and Command Palette visual checks

## Intentional 1.0 boundaries

- Git is read-only.
- PDF, video, and Office formats show metadata and defer full rendering/playback
  to their associated applications.
- Nimbus uses consistent type/folder icons; extracting every per-file Windows
  Shell icon is reserved for a later adapter revision.
- Specialized lock-owner inspection and extensible workflow plug-ins remain
  advanced Windows-workflow follow-ups rather than dependencies of core file
  navigation.
