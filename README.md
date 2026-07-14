# Nimbus

Nimbus is a hobby project for building a Rust-based Windows file explorer.

It is an experimental desktop application for Windows built with Rust and GPUI. The project is intended as a hands-on way to learn Rust GUI application architecture while implementing file explorer features such as basic navigation, search, file metadata, thumbnails, and Windows Shell integration.

## Goals

- Build a file explorer that feels natural on Windows
- Learn desktop GUI application structure with Rust
- Experiment with core explorer features such as file watching, search, thumbnails, and moving files to the recycle bin
- Document and refine Windows Shell API integration

## Tech Stack

- Rust 2024 Edition
- GPUI
- Windows API
- Tokio
- notify
- walkdir, jwalk

## Run

```powershell
cargo run
```

## Status

Nimbus is in the early development stage. The GPUI application skeleton and dependency setup are in place, and file explorer features will be implemented incrementally.
