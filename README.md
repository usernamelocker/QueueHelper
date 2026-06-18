# Queue Helper

A Windows desktop app that automates League of Legends champion select by talking to the LCU API. Handles ready checks, bans, picks, and hovers so you don't have to stare at champ select.

Built with Tauri 2 — Rust backend, React/TypeScript frontend.

## Features

- **Auto-Accept** — accepts ready checks with configurable random delay
- **Auto-Ban** — bans from your priority list during ban phase
- **Auto-Pick** — locks in from your priority list during pick phase
- **Auto-Hover** — hovers your pinned champion during planning
- **Draft Rules** — alerts on enemy/teammate picks, auto-switch profile by role
- **Queue Overrides** — different auto-accept behavior per queue type
- **Pick Position Request** — request a swap for desired pick position
- **System Tray** — minimize to tray, left-click to show/hide
- **Profiles** — named champion lists with drag-and-drop reorder, pin for hover, lock picks
- **Monitor Log** — real-time log of what the app is doing
- **i18n** — English and Turkish

## Getting Started

### Option 1: Download

Grab `QueueHelper_Setup.exe` from the Releases page. Double-click to install.

### Option 2: Build from source

```
git clone https://github.com/usernamelocker/QueueHelper
npm install
npm run tauri build
```

The installer will be at `src-tauri/target/release/bundle/nsis/QueueHelper_Setup.exe`.

### Usage

1. Open League of Legends client
2. Launch Queue Helper — it auto-connects
3. Enable features on the Dashboard
4. Queue up

## Development

```
npm run tauri dev      # dev mode with hot reload
npm run tauri build    # production build
npm run dev            # frontend only (needs backend running)
cargo check            # check Rust compilation
```

The backend logs LCU HTTP requests with `[LCU-HTTP]` prefix in debug builds.

## Settings

Each automation has adjustable delay with random jitter. Queue overrides let you disable auto-accept for specific game modes.

## Profiles

Drag-drop to reorder champions per profile. Pin one for auto-hover. Lock a pick to ignore teammate hovers. Draft Rules can auto-switch profiles based on your assigned role.

## Tech Stack

- **Backend:** Rust, Tauri 2, tokio, reqwest, tokio-tungstenite, rusqlite
- **Frontend:** React 19, TypeScript, Tailwind CSS v4, @dnd-kit
- **LCU:** HTTPS REST + WebSocket events

## Data

Settings, profiles, rules, and monitor logs live in `%APPDATA%\com.admin.queue-helper\`.

## Disclaimer

This app uses the official LCU API — same one the client uses. No memory injection or modifications. Use at your own risk.
