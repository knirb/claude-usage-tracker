# Claude Usage Tracker

A macOS menubar widget that shows your real-time Claude Code usage limits — the same data you see with `/usage` or on the web dashboard.

Click the tray icon to see your current session and weekly usage at a glance.

## Features

- **Menubar-only** — no Dock icon, lives in your tray
- **Three usage meters** — current session (5-hour), weekly all models, and weekly Sonnet
- **Auto-refresh** — polls every 5 minutes, or click to refresh manually
- **Auto-dismiss** — popup hides when you click away
- **Secure** — OAuth tokens stay in the Rust backend, never exposed to the WebView

## Prerequisites

- macOS 10.15+
- [Claude Code](https://claude.ai/claude-code) signed in (the app reads your OAuth token from the macOS Keychain)

## Build from source

Requires [Rust](https://rustup.rs/) and [Node.js](https://nodejs.org/).

```sh
git clone https://github.com/knirb/claude-usage-tracker.git
cd claude-usage-tracker
npm install
npx tauri build
```

The `.app` bundle will be at `src-tauri/target/release/bundle/macos/Claude Usage Tracker.app`.

## How it works

The app reads your Claude Code OAuth credentials from the macOS Keychain and calls the same usage API endpoint that powers the `/usage` command and web dashboard. All HTTP requests happen in the Rust backend — your tokens never touch the frontend.

## Tech stack

- [Tauri v2](https://tauri.app/) — Rust backend + WebView frontend
- Vanilla HTML/CSS/JS — no framework needed
- [security-framework](https://crates.io/crates/security-framework) — macOS Keychain access
- [reqwest](https://crates.io/crates/reqwest) — HTTP client with rustls
