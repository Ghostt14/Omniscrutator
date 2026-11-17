# Omniscrutator

A Raycast-style front end for voidtools' Everything search tool. Still
phase 1 of 3 (layout + interaction skeleton, running against a fake
in-memory dataset) -- but now with a modern toolchain, CI, and the behavior
changes below.

## What changed in this pass
- **Renamed** project/binary/window to `Omniscrutator`.
- **No visible filter tabs.** Search is always unfiltered ("All"). What
  category gets sorted to the *front* of results is now a Settings-only
  preference (`ResultPriority`: General / Apps first / Folders first /
  Documents first / Images first) -- a dropdown in the settings page, not a
  toggle on the main screen.
- **Settings access, no button:** type `settings` (exact match) and an
  "Open Settings" row appears -- press Enter to open it. There's no gear
  icon or keyboard shortcut on the main screen anymore.
  "Right-click the tray icon" isn't possible yet because there's no tray
  icon (that's Win32-specific, future work) -- as an interim stand-in,
  right-clicking **anywhere in the window** also opens Settings. Swap this
  for a real tray context menu once the app has a tray icon.
- **Quick math:** type an arithmetic expression -- `4(5)-3`, `2^10`,
  `(2+3)*4` -- and the evaluated result is shown as the very first row,
  ahead of any file search or web-search prompt. Implicit multiplication
  (`4(5)` = `4*5`) is supported. This is hand-rolled in `src/mathexpr.rs`
  (no external crate) with unit tests -- run `cargo test`.
- **Toolchain unpinned.** The first pass was pinned to old egui 0.19 /
  `url`=2.4.1 because the sandbox that wrote it only had rustc 1.75 via
  apt. This pass installed a current Rust (1.100, via conda-forge) and
  bumped back to egui/eframe 0.29 -- verified compiling, testing, running,
  and rendering correctly (screenshots were taken from a real run, not
  mocked up).

## CI
`.github/workflows/build.yml` runs on every push/PR:
- `check` (ubuntu-latest): `cargo check` + `cargo test` -- fast feedback,
  works because `MockBackend` is cross-platform and the real Windows IPC
  code is behind `#[cfg(windows)]`.
- `build-windows` (windows-latest): full `cargo build --release`, uploads
  `omniscrutator.exe` as a build artifact.

Both jobs use `dtolnay/rust-toolchain@stable`, so CI always builds with
whatever the current stable Rust is -- no version pinning needed there.

## What's here
- `src/main.rs` -- window setup.
- `src/app.rs` -- the UI: search bar, result list (keyboard nav), the
  math/command row injection, settings panel.
- `src/mathexpr.rs` -- the arithmetic evaluator, with unit tests.
- `src/everything.rs` -- `EverythingBackend` trait + `MockBackend`
  (used everywhere for now) + `ResultPriority` (settings-only sort
  preference) + the stubbed `EverythingIpcBackend` (`#[cfg(windows)]`,
  `TODO`s for the real IPC/FFI calls into Everything64.dll -- phase 2).
- `src/settings.rs` -- settings struct, saved as
  `omniscrutator_settings.json` next to the exe (portable, no registry).

## Building locally
```
cargo build            # debug
cargo build --release  # optimized, smaller binary
cargo test              # math-evaluator unit tests
cargo run
```

## Known rough edges (expected at this stage)
- No real icons yet -- `[FILE]`/`[DIR]`/`[=]`/`[>]` text placeholders,
  since egui's default font has no emoji glyphs.
- The status bar's left (backend status) and right (hint text) labels can
  overlap at narrow widths -- needs a proper two-column layout or truncation.
- Window is normal chrome (resizable, decorated) for easy dragging while
  iterating. The "hold hotkey, borderless popup under cursor" feel is a
  deliberate phase-2/3 item.
- Activating a file / web-search-on-Enter currently just report what they
  *would* do in the status bar -- no `ShellExecute` or browser launch yet.
- Right-click-to-open-settings fires anywhere in the window, including on
  top of result rows -- fine for now since there's no per-row context menu
  yet, but will need scoping once there is one.

## Idle RAM reality check
Measured ~120MB RSS for the earlier (0.19) build under Xvfb on Linux
(OpenGL context + font atlas + winit/glutin overhead); expect something in
that neighborhood here too. That's normal for an always-rendered egui/glow
window -- it will not get down to single-digit MB. A near-zero idle
footprint needs an architecture split:
1. A tiny resident background piece (global hotkey registration + tray
   icon only, no GUI framework) -- "a couple MB idle" is realistic here.
2. The egui search window, spawned on hotkey-press and torn down (not
   just hidden) when dismissed, so its GL context/memory is freed between
   uses.
This is phase-3 ("reduce hardware needs") restructuring, flagged now so
it's not a surprise later.

## Next steps (in order)
1. You look at this, tell me what to change about the layout/feel.
2. Wire up `EverythingIpcBackend` for real (Everything's documented IPC
   window protocol or `Everything64.dll` via `LoadLibraryW`/`GetProcAddress`).
3. Global hotkey + borderless popup window + real tray icon with a right-
   click context menu (replacing the window-wide right-click interim).
4. `ShellExecute` on Enter, real clipboard copy-path, web-search URL launch.
5. Split into resident-listener + on-demand-window per the RAM note above.
