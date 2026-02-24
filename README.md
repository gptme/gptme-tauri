# gptme-tauri

A desktop application for [gptme](https://github.com/gptme/gptme) built with [Tauri](https://tauri.app/).

This app packages the gptme web UI (bundled in the [gptme](https://github.com/gptme/gptme) repo) with a bundled `gptme-server` binary, providing a standalone desktop experience for gptme.

## Features

- 🖥️ Native desktop app with web UI
- 📦 Self-contained with bundled gptme-server
- 🚀 No need to install Python or manage dependencies
- 🔧 All gptme tools and capabilities available

## Prerequisites

- [Node.js](https://nodejs.org/) (for building)
- [Rust](https://rustup.rs/) (for Tauri)
- Git submodules initialized

## Development

```bash
# Install dependencies and initialize submodules
git submodule update --init --recursive
npm install

# Run in development mode
make dev
```

## Building

```bash
# Build the application
make build
```

The built application will be in `src-tauri/target/release/bundle/`.

## Project Structure

- `gptme/` - gptme source code (submodule, includes webui at `gptme/webui/`)
- `src-tauri/` - Tauri backend configuration
- `bins/` - Contains the bundled gptme-server binary
