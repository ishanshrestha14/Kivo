# macOS Build Support — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add macOS build support to Kivo with separate Intel/Apple Silicon DMGs and a polished transparent titlebar.

**Architecture:** Configure Tauri 2 to produce macOS DMG bundles, generate a proper `.icns` icon from the existing 500x500 source, enable the overlay titlebar style in the window config, and add CSS + JS platform detection so the transparent titlebar only takes effect on macOS (leaving Windows/Linux unchanged).

**Tech Stack:** Tauri 2, React 19, Tailwind CSS, macOS `sips`/`iconutil` for icon generation

---

### Task 1: Generate macOS Icon (.icns)

**Files:**
- Create: `desktop/icons/icon.icns`
- Source: `assests/icon/icon.png` (500x500 — sufficient for all sizes up to 512@2x with nearest-neighbor upscale)

- [ ] **Step 1: Create the .iconset directory with all required sizes**

Run this script to generate all icon sizes from the 500x500 source PNG using macOS `sips`:

```bash
cd /Users/sri/Developer/OpenSource/Kivo

mkdir -p desktop/icons/Kivo.iconset

# Generate each required size
sips -z 16 16     assests/icon/icon.png --out desktop/icons/Kivo.iconset/icon_16x16.png
sips -z 32 32     assests/icon/icon.png --out desktop/icons/Kivo.iconset/icon_16x16@2x.png
sips -z 32 32     assests/icon/icon.png --out desktop/icons/Kivo.iconset/icon_32x32.png
sips -z 64 64     assests/icon/icon.png --out desktop/icons/Kivo.iconset/icon_32x32@2x.png
sips -z 128 128   assests/icon/icon.png --out desktop/icons/Kivo.iconset/icon_128x128.png
sips -z 256 256   assests/icon/icon.png --out desktop/icons/Kivo.iconset/icon_128x128@2x.png
sips -z 256 256   assests/icon/icon.png --out desktop/icons/Kivo.iconset/icon_256x256.png
sips -z 512 512   assests/icon/icon.png --out desktop/icons/Kivo.iconset/icon_256x256@2x.png
sips -z 512 512   assests/icon/icon.png --out desktop/icons/Kivo.iconset/icon_512x512.png
sips -z 512 512   assests/icon/icon.png --out desktop/icons/Kivo.iconset/icon_512x512@2x.png
```

Note: The source is 500x500, so the 512x512 sizes will be slightly upscaled. This is acceptable — `sips` handles it gracefully. For pixel-perfect results, a 1024x1024 source would be ideal, but 500x500 is fine for this project.

Expected: 10 PNG files in `desktop/icons/Kivo.iconset/`

- [ ] **Step 2: Convert the iconset to .icns**

```bash
iconutil -c icns desktop/icons/Kivo.iconset -o desktop/icons/icon.icns
```

Expected: `desktop/icons/icon.icns` is created (typically ~50-200 KB)

- [ ] **Step 3: Clean up the intermediate iconset directory**

```bash
rm -rf desktop/icons/Kivo.iconset
```

- [ ] **Step 4: Verify the .icns file**

```bash
file desktop/icons/icon.icns
```

Expected output: `desktop/icons/icon.icns: Mac OS X icon, ...`

- [ ] **Step 5: Commit**

```bash
git add desktop/icons/icon.icns
git commit -m "feat(macos): generate .icns icon from source PNG"
```

---

### Task 2: Configure Tauri for macOS Builds

**Files:**
- Modify: `desktop/tauri.conf.json`

- [ ] **Step 1: Add macOS bundle targets and icon**

In `desktop/tauri.conf.json`, update the `bundle` section:

Change the `targets` array from:
```json
"targets": ["msi", "nsis", "deb"]
```
To:
```json
"targets": ["msi", "nsis", "deb", "dmg", "app"]
```

Add `"icons/icon.icns"` to the `icon` array:
```json
"icon": [
  "icons/icon.ico",
  "icons/icon.png",
  "icons/icon.icns"
]
```

- [ ] **Step 2: Add macOS-specific bundle configuration**

Add a `"macOS"` section inside `bundle` (sibling to `"windows"` and `"linux"`):

```json
"macOS": {
  "minimumSystemVersion": "10.15",
  "dmg": {
    "appPosition": { "x": 180, "y": 170 },
    "applicationFolderPosition": { "x": 480, "y": 170 }
  }
}
```

- [ ] **Step 3: Configure the transparent titlebar on the main window**

In the `app.windows[0]` object (the `"main"` window), add these two properties:

```json
{
  "label": "main",
  "title": "Kivo",
  "width": 1180,
  "height": 720,
  "resizable": true,
  "titleBarStyle": "overlay",
  "hiddenTitle": true
}
```

`titleBarStyle: "overlay"` renders the macOS traffic lights (close/minimize/maximize) on top of the webview content. `hiddenTitle: true` hides the centered "Kivo" text from the titlebar. These settings only take effect on macOS — Windows and Linux ignore them.

- [ ] **Step 4: Verify the final tauri.conf.json is valid JSON**

```bash
cd /Users/sri/Developer/OpenSource/Kivo
python3 -c "import json; json.load(open('desktop/tauri.conf.json')); print('Valid JSON')"
```

Expected: `Valid JSON`

- [ ] **Step 5: Commit**

```bash
git add desktop/tauri.conf.json
git commit -m "feat(macos): add DMG bundle target and transparent titlebar config"
```

---

### Task 3: Add macOS Platform Detection in App.jsx

**Files:**
- Modify: `src/app/App.jsx`

This task adds a `macos` class to `<body>` when running on macOS, so CSS can scope macOS-only styles. Uses `navigator.userAgent` — no extra dependency needed.

- [ ] **Step 1: Add platform detection useEffect**

In `src/app/App.jsx`, inside the `App` component function (after the existing `useEffect` for `resolvedPath` around line 91-98), add:

```jsx
useEffect(() => {
  if (/Macintosh/.test(navigator.userAgent)) {
    document.body.classList.add("macos");
  }
}, []);
```

This runs once on mount. The `Macintosh` string appears in the user agent on all macOS versions in the Tauri webview. On Windows/Linux, this is a no-op.

- [ ] **Step 2: Verify the change doesn't break the existing code**

```bash
cd /Users/sri/Developer/OpenSource/Kivo && pnpm frontend:build
```

Expected: Build succeeds with no errors.

- [ ] **Step 3: Commit**

```bash
git add src/app/App.jsx
git commit -m "feat(macos): add platform detection to scope macOS-only styles"
```

---

### Task 4: Add macOS Titlebar CSS Styles

**Files:**
- Modify: `src/index.css`

- [ ] **Step 1: Add macOS titlebar styles at the end of the `@layer base` block**

In `src/index.css`, add the following styles inside the `@layer base { ... }` block (after the `#root` rule at line 136-138, before the closing `}` of `@layer base`):

```css
  /* macOS transparent titlebar — only applies when body has .macos class */
  body.macos #root {
    padding-top: 28px;
  }
```

This adds top padding to the entire app root when on macOS, pushing all content below the titlebar overlay zone (where the traffic lights sit). The `28px` matches the macOS titlebar height in Tauri's overlay mode.

- [ ] **Step 2: Add drag region styles in the `@layer utilities` block**

At the end of the `@layer utilities { ... }` block (after the `.thin-scrollbar` rules, before the closing `}`), add:

```css
  /* macOS window drag region — makes the marked area draggable */
  [data-tauri-drag-region] {
    -webkit-app-region: drag;
  }

  [data-tauri-drag-region] button,
  [data-tauri-drag-region] input,
  [data-tauri-drag-region] a {
    -webkit-app-region: no-drag;
  }
```

The second rule ensures interactive elements within the drag region remain clickable — without it, buttons inside the drag zone would start a window drag instead of firing their click handler.

- [ ] **Step 3: Verify the CSS compiles**

```bash
cd /Users/sri/Developer/OpenSource/Kivo && pnpm frontend:build
```

Expected: Build succeeds with no errors.

- [ ] **Step 4: Commit**

```bash
git add src/index.css
git commit -m "feat(macos): add titlebar padding and drag region CSS"
```

---

### Task 5: Add Drag Region to Sidebar and Main Header

**Files:**
- Modify: `src/components/workspace/Sidebar.jsx:836-841`
- Modify: `src/app/App.jsx:224,281`

- [ ] **Step 1: Add drag region to the sidebar icon column**

In `src/components/workspace/Sidebar.jsx`, line 837, the sidebar's left icon column is:

```jsx
<Card className="flex min-h-0 flex-col items-center gap-2 bg-[hsl(var(--sidebar))]/96 p-2.5 shadow-none">
```

Add `data-tauri-drag-region` to this element:

```jsx
<Card data-tauri-drag-region className="flex min-h-0 flex-col items-center gap-2 bg-[hsl(var(--sidebar))]/96 p-2.5 shadow-none">
```

This makes the top area of the sidebar icon column draggable on macOS. The CSS rule from Task 4 ensures the buttons inside it remain clickable via `no-drag`.

- [ ] **Step 2: Add drag region to the main content header bars**

In `src/app/App.jsx`, there are two header bar `<div>` elements for the collection settings view and the workspace view. Both need `data-tauri-drag-region` so the window can be dragged from the header area.

**Collection settings header (line 224):**

Change:
```jsx
<div className="flex shrink-0 items-center justify-between border-b border-border/25 bg-background/40 px-5 py-3 backdrop-blur-md">
```
To:
```jsx
<div data-tauri-drag-region className="flex shrink-0 items-center justify-between border-b border-border/25 bg-background/40 px-5 py-3 backdrop-blur-md">
```

**Workspace view header (line 281):**

Change:
```jsx
<div className="flex shrink-0 items-center justify-between border-b border-border/25 bg-background/40 px-5 py-3.5 backdrop-blur-md">
```
To:
```jsx
<div data-tauri-drag-region className="flex shrink-0 items-center justify-between border-b border-border/25 bg-background/40 px-5 py-3.5 backdrop-blur-md">
```

- [ ] **Step 3: Verify the frontend still builds**

```bash
cd /Users/sri/Developer/OpenSource/Kivo && pnpm frontend:build
```

Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/components/workspace/Sidebar.jsx src/app/App.jsx
git commit -m "feat(macos): add drag regions to sidebar and header bars"
```

---

### Task 6: Install Intel Cross-Compilation Target

**Files:** None (Rust toolchain change only)

- [ ] **Step 1: Add the x86_64-apple-darwin target**

```bash
rustup target add x86_64-apple-darwin
```

Expected: `info: component 'rust-std' for target 'x86_64-apple-darwin' is up to date` (or downloads it)

- [ ] **Step 2: Verify both targets are available**

```bash
rustup target list --installed | grep apple
```

Expected output includes both:
```
aarch64-apple-darwin
x86_64-apple-darwin
```

No commit needed — this is a local toolchain change.

---

### Task 7: Build and Verify Apple Silicon DMG

**Files:** None (build verification)

- [ ] **Step 1: Run the full Tauri build**

```bash
cd /Users/sri/Developer/OpenSource/Kivo && pnpm build
```

This runs the frontend build (Vite) and the Rust build (Cargo), then bundles into `.app` and `.dmg`. First build takes 3-5 minutes.

Expected: Build completes without errors. Output in `desktop/target/release/bundle/`.

- [ ] **Step 2: Verify the DMG was created**

```bash
ls -la desktop/target/release/bundle/dmg/
```

Expected: A file like `Kivo_0.3.3_aarch64.dmg`

- [ ] **Step 3: Verify the .app bundle exists**

```bash
ls -la desktop/target/release/bundle/macos/
```

Expected: A `Kivo.app` directory

- [ ] **Step 4: Open the DMG and verify it mounts correctly**

```bash
open desktop/target/release/bundle/dmg/Kivo_0.3.3_aarch64.dmg
```

Expected: DMG mounts in Finder showing the Kivo app and an Applications folder shortcut.

- [ ] **Step 5: Launch the app and verify the transparent titlebar**

Open `Kivo.app` from the mounted DMG. Verify:
- Traffic lights (close/minimize/maximize) appear overlaid on the sidebar
- The sidebar and main content have proper padding below the traffic lights
- The window can be dragged from the sidebar icon column and header bars
- Buttons in the drag regions are still clickable
- The app icon appears correctly in the Dock

No commit needed — this is a verification step.

---

### Task 8: Build and Verify Intel DMG (Cross-Compile)

**Files:** None (build verification)

- [ ] **Step 1: Build for Intel target**

```bash
cd /Users/sri/Developer/OpenSource/Kivo && pnpm build -- --target x86_64-apple-darwin
```

Expected: Build completes. May show warnings about cross-compilation but should succeed.

- [ ] **Step 2: Verify the Intel DMG was created**

```bash
ls -la desktop/target/x86_64-apple-darwin/release/bundle/dmg/
```

Expected: A file like `Kivo_0.3.3_x64.dmg`

- [ ] **Step 3: Verify the binary architecture**

```bash
file desktop/target/x86_64-apple-darwin/release/bundle/macos/Kivo.app/Contents/MacOS/Kivo
```

Expected: `... Mach-O 64-bit executable x86_64`

No commit needed — this is a verification step.

---

### Task 9: Final Commit and Push

**Files:** None (git operations)

- [ ] **Step 1: Check overall status**

```bash
cd /Users/sri/Developer/OpenSource/Kivo && git status && git log --oneline feature/macos-build ^main
```

Expected: Clean working tree. Should show commits for tasks 1-5.

- [ ] **Step 2: Push to origin**

```bash
git push origin feature/macos-build
```

Expected: Branch pushed to `sriannamalai/Kivo`.
