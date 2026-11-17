# rovyl-search -- skeleton

A Raycast-style front end for voidtools' Everything search tool. This is
phase 1 of 3: **layout skeleton**, running against a fake in-memory dataset
(`MockBackend` in `src/everything.rs`) so the UI can be judged on its own
before any Windows- or Everything-specific code is written.

## What's here
- `src/main.rs` -- window setup.
- `src/app.rs` -- the whole UI: search bar, filter tabs (All/Apps/Folders/
  Docs/Images), result list with keyboard nav, settings panel.
- `src/everything.rs` -- `EverythingBackend` trait. `MockBackend` is used by
  default everywhere right now. `EverythingIpcBackend` is a stub, compiled
  only `#[cfg(windows)]`, with the real IPC/FFI calls into Everything64.dll
  left as `TODO`s -- that's phase 2.
- `src/settings.rs` -- settings struct, saved as `rovyl_settings.json` next
  to the exe (portable, no registry).

## Building
Needs a reasonably recent Rust (this skeleton was pinned to egui/eframe
0.19 + `url`/`idna` pins purely because the sandbox that wrote this only had
rustc 1.75 via apt, which can't build the newer `icu_*` crates that
current `eframe` pulls in transitively).

**On your own machine, with a normal `rustup`-installed toolchain (1.80+),
you should bump this back up**: in `Cargo.toml`, set `eframe`/`egui` back to
a current version (0.27+) and delete the `url`/`idna` pin lines -- you'll
get a nicer API (closure-based `ctx.input(|i| ...)`, `ViewportBuilder`,
proper borderless/transparent window support) and the Ctrl+, settings
shortcut can come back (egui 0.19's `Key` enum has no comma key, which is
why F2 is used here instead).

```
cargo run              # debug build, window chrome shown
cargo build --release  # optimized, smaller binary
```

## Known rough edges (expected at this stage)
- No real icons yet -- `[FILE]`/`[DIR]` text placeholders, since egui's
  default font has no emoji glyphs. Real file-type icons come once we're
  past layout.
- The status bar's left (backend status) and right (hint text) labels can
  overlap at narrow widths -- needs a proper two-column layout or truncation.
- Window is normal chrome (resizable, decorated) for easy dragging while
  iterating. The actual "hold hotkey, borderless popup under cursor" feel
  is a deliberate phase-2/3 item, not attempted here.
- `open_selected` / web-search-on-Enter currently just report what they
  *would* do in the status bar -- no `ShellExecute` or browser launch yet.

## Idle RAM reality check
Measured ~120MB RSS for this skeleton under Xvfb on Linux (OpenGL context +
font atlas + winit/glutin overhead). That's normal for an always-rendered
egui/glow window -- it will not get down to single-digit MB. If a near-zero
idle footprint matters, the architecture should split into:
1. A tiny resident background piece (global hotkey registration + tray icon
   only, no GUI framework) -- this is where "a couple MB idle" is realistic.
2. The egui search window, spawned on hotkey-press and torn down (not just
   hidden) when dismissed, so its GL context/memory is freed between uses.

This is a phase-3 ("reduce hardware needs") restructuring, not a phase-1
concern -- flagging it now so it's not a surprise later.

## Next steps (in order)
1. You look at this, tell me what to change about the layout/feel.
2. Wire up `EverythingIpcBackend` for real (Everything's documented IPC
   window protocol or `Everything64.dll` via `LoadLibraryW`/`GetProcAddress`).
3. Global hotkey + borderless popup window + tray icon (Win32-specific).
4. `ShellExecute` on Enter, real clipboard copy-path, web-search URL launch.
5. Split into resident-listener + on-demand-window per the RAM note above.
