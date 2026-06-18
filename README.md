# Queue Helper

Windows desktop app that automates League of Legends champion select. Connects to the LCU (League Client Update) API and handles the boring parts of champ select so you don't have to stare at the screen waiting for a ban or pick to time out.

Built with Tauri 2 (Rust backend, React/TypeScript frontend).

## Features

- **Auto-Accept** — accepts ready checks automatically. Has a random delay so it doesn't look bot-ish.
- **Auto-Ban** — bans champions from your priority list during the ban phase.
- **Auto-Pick** — locks in champions from your priority list during pick phase.
- **Auto-Hover** — hovers your pinned champion during the planning phase.
- **Draft Rules** — shows alerts when certain champs are picked/banned by enemies or teammates. Can auto-switch your active profile based on your assigned role.
- **Queue Overrides** — lets you set different auto-accept behavior per queue type. e.g. enable for Draft but disable for ARAM.
- **Pick Position Request** — requests a swap for your preferred pick position after banning.
- **System Tray** — minimizes to tray when you close the window. Click the tray icon to show/hide. Has a right-click menu too.
- **Profiles** — create named lists of champions with drag-and-drop reorder. Pin a champ for auto-hover. Lock a pick so it ignores what your teammates hover.
- **Monitor Log** — shows a real-time log of what the app is doing (bans, picks, hovers, etc.). Helps you see if something isn't working right.
- **Language Support** — English and Turkish. Pick it in Settings.

## Installation

Grab the latest installer from Releases:

- `QueueHelper_Setup.exe` — NSIS installer, double-click and done. No admin required.

Or build from source:

- `npm install`
- `npm run tauri build`
- Installer will be in `src-tauri/target/release/bundle/nsis/`

## How to Use

1. Have League of Legends running (client open, not in-game).
2. Launch Queue Helper. It auto-connects to the LCU.
3. Go to the Dashboard and toggle the features you want.
4. Join a queue. The app handles the rest.

The app reads your summoner info and the current game session from the LCU API. No injection, no memory reading, just API calls that the client exposes. Should be safe from a ban perspective (it's the same API the official client uses internally), but use at your own risk.

## Settings

Each automation feature has its own delay setting with random jitter. Tune these to whatever feels natural. Queue overrides let you disable auto-accept for specific queues.

## Profiles

Drag and drop to reorder champions. Each profile has:
- A list of champs (ordered by priority)
- A pinned champ for auto-hover
- A locked pick option (ignores teammate hover for that role)

You can have as many profiles as you want. Assign one per role from Draft Rules if you want auto-switching.

## Draft Rules

Simple rules engine. Each rule watches for a specific champion and triggers an alert in the monitor log. Can also auto-switch your active profile when you get assigned a specific role.

## i18n

Currently supports:
- English (default)
- Turkish

Language setting is persisted and syncs with the backend.

## Tech Stack

**Backend:** Rust, Tauri 2, tokio, reqwest, tokio-tungstenite, rusqlite, serde
**Frontend:** React 19, TypeScript, Tailwind CSS v4, @dnd-kit
**LCU Communication:** HTTPS REST (Basic auth via lockfile) + WebSocket for live events

## Development

```
npm run tauri dev      # dev mode with hot reload
npm run tauri build    # production build + installer
npm run dev            # frontend only (won't work alone)
cargo check            # check Rust compilation
```

The Rust backend logs LCU HTTP requests with a `[LCU-HTTP]` prefix when built in debug mode. Useful for figuring out what the app is seeing from the client.

## Notes

- First time setup creates a `settings.json`, profile database, and monitor log in `%APPDATA%\com.admin.queue-helper\`.
- The app needs the League client to be running. It won't do anything useful if the client isn't open.
- System tray icon uses your app icon. Left-click toggles the window, right-click shows a menu.
- If the app crashes or won't open, check the monitor log or the Rust stderr output.
