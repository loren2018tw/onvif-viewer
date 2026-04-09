# Project Guidelines

## Architecture

Tauri v2 desktop app — Rust backend + Vue 3 / Vuetify 3 frontend.

```
src-tauri/          # Rust backend (Tauri commands, ONVIF logic, storage)
  src/
    main.rs         # Tauri app entry
    lib.rs          # Module exports
    commands.rs     # #[tauri::command] handlers (frontend ↔ backend bridge)
    camera.rs       # oxvif session: get stream URI, device info
    discovery.rs    # WS-Discovery (oxvif::discovery::probe) + IP range scan
    models.rs       # Shared data types (CameraInfo, DiscoveredCamera, ScanRange)
    storage.rs      # JSON file persistence (~/.config/onvif-viewer/cameras.json)
frontend/           # Vue 3 + Vuetify 3 + TypeScript
  src/
    main.ts         # App bootstrap (Pinia, Router, Vuetify)
    App.vue         # Shell with app bar and theme toggle
    plugins/vuetify.ts
    router/index.ts
    stores/camera.ts    # Pinia store — all Tauri invoke() calls
    types/camera.ts     # TypeScript interfaces mirroring Rust models
    views/HomeView.vue  # Main UI: discovery panel + saved cameras + preview
```

### Key decisions

- **oxvif** crate (`0.8`) for all ONVIF operations — uses `OnvifSession` with clock sync for stream URI retrieval
- Camera credentials stored locally in `~/.config/onvif-viewer/cameras.json`
- Single-camera preview only (stream URI display + copy) — no embedded video player
- Frontend communicates with backend exclusively through `@tauri-apps/api/core` `invoke()`

## Build and Test

```bash
# Install frontend deps
cd frontend && pnpm install

# Dev mode (starts both Vite dev server and Tauri)
cd src-tauri && cargo tauri dev

# Production build
cd src-tauri && cargo tauri build

# Check Rust only
cd src-tauri && cargo check

# Build frontend only
cd frontend && npx vite build
```

**System dependency:** `libssl-dev` (Ubuntu/Debian) is required for `openssl-sys`.

## Code Style

- Rust: 2021 edition, async/await with tokio, `anyhow::Result` for error handling in internal code, `Result<T, String>` for Tauri commands
- TypeScript: strict mode, path alias `@/` → `src/`
- Vue: `<script setup lang="ts">` composition API, Vuetify components
- UI language: Traditional Chinese (zh-TW)

## Conventions

- Tauri commands in `commands.rs` are thin wrappers — business logic lives in `camera.rs`, `discovery.rs`, `storage.rs`
- Frontend types in `types/camera.ts` must mirror Rust `models.rs` (snake_case field names preserved for serde compatibility)
- All Tauri `invoke()` calls go through the Pinia store, not directly from components
