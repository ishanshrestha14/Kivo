# macOS Build Support for Kivo

**Date:** 2026-04-09
**Branch:** `feature/macos-build`
**Status:** Approved

## Goal

Add macOS build support to Kivo, producing signed DMG installers for both Intel (`x86_64-apple-darwin`) and Apple Silicon (`aarch64-apple-darwin`) architectures, with a polished native macOS window experience using a transparent titlebar.

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Architecture targets | Separate Intel + Apple Silicon DMGs | Smaller downloads, simpler build, standard Tauri approach |
| Window style | Transparent titlebar with overlay traffic lights | Native macOS feel (like Slack, Discord, Arc) |
| Code signing | Local signing via Xcode identity | User has Apple Development cert; Developer ID Application cert needed later for distribution |
| Notarization | Deferred | Requires Developer ID Application certificate |
| CI/CD | Deferred | Local builds only for this iteration |

## Changes

### 1. Tauri Configuration (`desktop/tauri.conf.json`)

**Bundle targets:** Add `"dmg"` and `"app"` to the existing `bundle.targets` array.

**macOS bundle config:** Add a `bundle.macOS` section:
- `minimumSystemVersion`: `"10.15"` (Catalina — Tauri 2 minimum)
- `signingIdentity`: `"-"` (ad-hoc signing; uses available Keychain identity automatically)
- `dmg.appPosition`: `{ "x": 180, "y": 170 }`
- `dmg.applicationFolderPosition`: `{ "x": 480, "y": 170 }`

**Icon:** Add `"icons/icon.icns"` to the `bundle.icon` array.

**Window titlebar:** On the main window object, add:
- `"titleBarStyle": "overlay"` — renders traffic lights over the webview
- `"hiddenTitle": true` — hides the centered window title text

These are Tauri-level settings and only take effect on macOS. Windows/Linux windows are unaffected.

### 2. macOS Icon (`desktop/icons/icon.icns`)

Generate from the existing `desktop/icons/icon.png` (17KB, sufficient resolution) using macOS `iconutil`:
- Create an `.iconset` directory with all required sizes (16, 32, 64, 128, 256, 512 at 1x and 2x)
- Run `iconutil -c icns` to produce the `.icns` file
- Add to the icons directory alongside existing `.ico` and `.png`

### 3. CSS — Titlebar Padding (`src/index.css`)

Add macOS-scoped styles that only activate when the `<body>` has class `macos`:

```css
/* macOS transparent titlebar support */
body.macos .sidebar-container {
  padding-top: 38px; /* Space for traffic light buttons */
}

body.macos .main-header {
  padding-top: 32px; /* Space for titlebar overlay zone */
}

[data-tauri-drag-region] {
  -webkit-app-region: drag;
}
```

These styles are inert on Windows/Linux since the `.macos` class is never added on those platforms.

### 4. Platform Detection (`src/app/App.jsx`)

On app mount, use `@tauri-apps/api/core` or `@tauri-apps/plugin-os` to detect the platform:

```js
import { platform } from '@tauri-apps/plugin-os';

// In useEffect on mount:
const os = platform();
if (os === 'macos') {
  document.body.classList.add('macos');
}
```

If `@tauri-apps/plugin-os` is not already a dependency, use Tauri's `navigator.userAgent` or `window.__TAURI__` platform detection as a zero-dependency alternative.

### 5. Drag Region (`src/components/workspace/Sidebar.jsx`)

Add `data-tauri-drag-region` attribute to the top area of the sidebar so the window can be dragged by the sidebar header on macOS. This attribute is a Tauri convention — it marks the element as a window drag handle.

The attribute is harmless on Windows/Linux (no effect).

## Build Process

```bash
# Apple Silicon (native on M-series Mac)
pnpm build

# Intel (cross-compile from Apple Silicon)
rustup target add x86_64-apple-darwin
pnpm build -- --target x86_64-apple-darwin
```

**Output locations:**
- `desktop/target/release/bundle/dmg/Kivo_0.3.3_aarch64.dmg`
- `desktop/target/x86_64-apple-darwin/release/bundle/dmg/Kivo_0.3.3_x64.dmg`

## Files Changed

| File | Change Type | Description |
|------|-------------|-------------|
| `desktop/tauri.conf.json` | Modified | Add dmg/app targets, macOS config, titlebar overlay |
| `desktop/icons/icon.icns` | New | macOS app icon generated from icon.png |
| `src/index.css` | Modified | macOS titlebar padding + drag region styles |
| `src/app/App.jsx` | Modified | Platform detection, add .macos class to body |
| `src/components/workspace/Sidebar.jsx` | Modified | Add data-tauri-drag-region to header |

## Out of Scope

- CI/CD pipeline changes (future iteration)
- Notarization (requires Developer ID Application certificate)
- Universal binary (fat binary combining both architectures)
- macOS App Store distribution
- macOS-specific keyboard shortcuts remapping (Cmd vs Ctrl — Tauri handles this automatically for standard shortcuts)
