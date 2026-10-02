# Flowform GPUI fork

## Purpose

This fork of Zed's GPUI supplies rendering and latency APIs used by the Flowform
native client. Keep this inventory when merging upstream Zed or changing the
client's GPUI dependency. Paths in the table are relative to this repository;
client call sites are relative to `flowform-client/src` in the Flowform monorepo.
The client's `Cargo.toml` patches GPUI crates to `../../zed-fork/crates/*`.

## Current base

- Fork `origin/main`: `ccc83bcc04e394a7cd83e38506d386f40528a9d4`.
- Upstream merge-base: `933d8d93819c749a607e561883855a9b95c79cea`
  (upstream Zed commit dated 2026-09-25). Merge commit `ccc83bcc04` brought
  that upstream revision onto fork main on 2026-09-25.
- Compare against the merged upstream revision, not the earlier July base
  `6b9f448ffc`. The fork-specific non-merge commits are still reachable after
  the September merge.

## Flowform-only commits

These are the non-merge commits in `933d8d9381..ccc83bcc04` absent from the
merged upstream history (oldest first):

| Commit | Date | Change |
| --- | --- | --- |
| `0394e05943` | 2026-05-29 | UV image transforms, video surfaces, aspect-aware layout. |
| `625af3347d` | 2026-06-10 | Rounded corners for painted surfaces. |
| `a2fc5f7ec4` | 2026-07-10 | macOS input-queue and GPU-present latency counters. |
| `00ebceccb4` | 2026-07-13 | Image/surface UV crop; fold object-fit overflow into crop. |
| `7c8a5ffbca` | 2026-07-14 | Correct transformed sprite sampling across renderers. |

The current tree, rather than each pre-merge diff, is authoritative: the
September merge relocated the shared Metal renderer into `gpui_apple`.

Windows parity commit `b926dfaa7d` (2026-10-02, on `3f8e42df2d`) restores three
Flowform-only Windows changes that the `flowform/upstream-2026-09-25` base
dropped because they lived only on `flowform/upstream-2026-07-14`
(`a6249944b3` dot-grid shader parity, `60f925cb30` Windows window-platform
support): the HLSL and WGSL `pattern_dots` case, the live-cursor window-control
hit test, and Windows `start_window_move` / `titlebar_double_click`. Without
them Windows canvases paint solid black and the custom title bar cannot be
dragged. Carry this commit through every upstream merge.

## Flowform-only APIs and behavior

| API / behavior | Fork files | Client call sites | Backends |
| --- | --- | --- | --- |
| `ContentMask::new(bounds)` and `ContentMask::rounded(bounds, corner_radii)`; `rounded_bounds`/`corner_radii` retain the original arc independently of the intersected rectangular `bounds`. `Style::overflow_mask` uses the element's radii when both axes hide overflow. | `crates/gpui/src/window.rs`, `crates/gpui/src/style.rs`; shader files below | `app/board_view/canvas.rs`, `views/dot_grid.rs` | Metal: antialiased clips on quads, shadows, underlines, paths, monochrome/polychrome sprites and surfaces. HLSL/WGSL: rounded fragment discard on quads, shadows, underlines and all sprites; paths remain rectangular. |
| `PolychromeSprite::pad2` explicitly aligns the record's tail for WGSL storage arrays. With the rounded mask the record is 152 bytes (38 words), identical across Rust, Metal, native WGSL, WebGL and HLSL. | `crates/gpui/src/scene.rs`, shader files and `crates/gpui_wgpu/src/wgpu_renderer.rs` layout tests | Indirect via image elements | All backends share the same record layout. |
| `gpui::pattern_dots(color, spacing, radius)` returns a `Background` with `BackgroundTag::Dots = 4`; the style fallback uses its solid color. Spacing and radius are screen-space/device-pixel values. | `crates/gpui/src/color.rs`, `crates/gpui/src/style.rs`, `crates/gpui_apple/src/shaders.metal` (`prepare_fill_color`, `fill_color`), `crates/gpui_windows/src/shaders.hlsl` and `crates/gpui_wgpu/src/shaders.wgsl` (`prepare_gradient_color`, `gradient_color` tag 4) | `views/dot_grid.rs` | Metal, HLSL and WGSL dot lattice. A backend without the tag-4 case paints an opaque rectangle (the Windows black canvas). |
| `gpui::ImageTransform { rotation_quarters, flip_h, flip_v }`, `encode()` and `swaps_axes()`; rotation is quarter turns and the packed UV bits are 0–3. | `crates/gpui/src/elements/img.rs`, `crates/gpui/src/scene.rs`; shader files below | `app/primitive/canvas.rs`, `app/crop.rs` | Image sprites: Metal, WGSL, HLSL. |
| `Img::transform(ImageTransform)` and `Img::crop(Bounds<f32>)`; crop is normalized in displayed (post-transform) orientation. Layout swaps intrinsic axes for 90°/270° rotation; object-fit uses rotated and cropped dimensions. Out-of-bounds crops are clamped; zero-area/identity crops are ignored by `sanitize_crop`. | `crates/gpui/src/elements/img.rs`, `crates/gpui/src/window.rs` | Transform: `app/primitive/canvas.rs`, `app/crop.rs`; crop: `app/primitive/canvas.rs`, `app/grid_view/media.rs` | Image sprites: Metal, WGSL, HLSL. |
| Cover/None image overflow is folded into the displayed-orientation UV crop instead of an atlas sub-tile. The visible quad gets the corner radii. | `crates/gpui/src/elements/img.rs` (`fold_overflow_into_crop`), `crates/gpui/src/window.rs` | Image cards in `app/primitive/canvas.rs`; video thumbnails in `app/grid_view/media.rs` | Image sprites: Metal, WGSL, HLSL. |
| `Surface::crop(Bounds<f32>)` uses normalized UVs; the cropped region determines object-fit size. The element style supplies corner radii, and Cover overflow is folded into crop. Surface content is `CVPixelBuffer`. | `crates/gpui/src/elements/surface.rs`, `crates/gpui/src/window.rs`, `crates/gpui_apple/src/metal_renderer.rs`, `crates/gpui_apple/src/shaders.metal` | `app/grid_view/media.rs` (decoded video frame) | macOS/Metal only. |
| `Window::paint_image(bounds, image_bounds, corner_radii, data, frame_index, grayscale, uv_transform, crop)` and macOS `Window::paint_surface(bounds, corner_radii, image_buffer, crop)` pass the UV crop and rounded bounds to the scene. | `crates/gpui/src/window.rs` | Indirectly through `img` in `app/primitive/canvas.rs`, `app/grid_view/media.rs`, `app/crop.rs`, and `surface` in `app/grid_view/media.rs`; no direct client calls. | Images: Metal/WGSL/HLSL; surfaces: macOS/Metal. |
| `scene::PolychromeSprite` has `uv_transform: u32` and `crop: [f32; 4]`; `scene::PaintSurface` has `corner_radii` and `crop`. The image crop defaults to `[0, 0, 1, 1]`. | `crates/gpui/src/scene.rs` | Indirect via image/video elements above; no direct scene construction. | Image scene data consumed by Metal/WGSL/HLSL; surface data by Metal. |
| `gpui_macos::input_latency::{LatencyCounter, INPUT_QUEUE_AGE, GPU_PRESENT}`; `drain()` returns and resets `(sum, max, count)`. Queue age samples pressed-button mouse moves and scrolls; GPU present measures Metal draw entry (including `next_drawable`) through command-buffer completion. | `crates/gpui_apple/src/input_latency.rs`, `crates/gpui_apple/src/gpui_apple.rs`, `crates/gpui_apple/src/metal_renderer.rs`, `crates/gpui_macos/src/gpui_macos.rs`, `crates/gpui_macos/src/window.rs` | `services/metrics.rs` | macOS event loop and shared Apple Metal renderer; exported through `gpui_macos`. |
| Metal `SurfaceBounds { corner_radii, crop }` (no `Eq`) and surface vertex/fragment crop, rounded SDF; polychrome vertex crops then inversely remaps transformed UVs. | `crates/gpui_apple/src/metal_renderer.rs`, `crates/gpui_apple/src/shaders.metal` | Indirect through `app/grid_view/media.rs` and `app/primitive/canvas.rs` | macOS/Metal. |
| WGSL and HLSL `PolychromeSprite` add `uv_transform` and crop; vertex shaders crop before applying inverse rotation/flip. WebGL's fixed decoder uses a 38-word sprite stride, matching Rust's 152-byte record. | `crates/gpui_wgpu/src/shaders.wgsl`, `crates/gpui_wgpu/src/shaders_webgl.wgsl`, `crates/gpui_wgpu/src/wgpu_renderer.rs`, `crates/gpui_windows/src/shaders.hlsl` | Indirect through image callers above. | wgpu/WebGL and Windows/HLSL image paths. |
| Windows title bar: `start_window_move` (via `SC_MOVE`) and `titlebar_double_click` (maximize/restore when resizable) are implemented, and window-control hit testing uses the live cursor position instead of the cached one. The client lets Windows run the native caption move loop and only registers a drag control area. | `crates/gpui_windows/src/window.rs`, `crates/gpui/src/window.rs` | `views/shell.rs` | Windows only; macOS ignores the hit-test callback. |

The image sampling contract is worth checking independently of compilation:

- `ImageTransform::encode()` uses bits 0–1 for clockwise rotation quarters,
  bit 2 for horizontal flip, and bit 3 for vertical flip. Sprite shaders map
  output UVs back to source UVs with the inverse rotation.
- Both `Img::crop` and `Surface::crop` accept normalized `Bounds<f32>`.
  `sanitize_crop` clamps them to the unit square, drops zero-area/identity
  rectangles, and passes `[x, y, width, height]` into the scene.
- Cropping is expressed in the displayed orientation. The shaders apply that
  crop **before** rotation/flip, while the element's object-fit calculation
  sizes the visible region rather than the original texture.
- `Window::paint_image` intersects image and visible bounds, folds overflow
  into the crop, then paints one quad with radii clamped to its visible size.
  `Surface::paint` makes the same visible-bounds adjustment before calling
  `paint_surface`; both preserve rounded visible corners under Cover fit.
- The dot-grid Metal shader anchors the lattice to the painted quad's origin.
  The client scales spacing/radius to device pixels and phases the quad with
  pan in `views/dot_grid.rs`.

## Rounded content-mask contract

- Zero radii mean no rounded constraint. Constructors clamp finite, nonnegative
  radii to half the shortest side; invalid radii become zero.
- Rectangular descendants intersect only `bounds`, preserving the parent's
  original `rounded_bounds` and circle centers. Scaling, snapped paint masks,
  split border strips and underline exclusions also preserve the rounded shape.
- A rectangle intersected with a rounded rectangle is exact. Coincident rounded
  bounds use the maximum radius at each corner; fully contained masks are exact.
  Two genuinely crossing rounded rectangles cannot always fit this single-curve
  representation: the intersection keeps the parent's curve and clips to a
  centered inscribed rectangle of the child. This is conservative (no leakage),
  but can over-clip crossing rounded children. It is not an arbitrary clip stack.
- Scene culling and hit testing still use rectangular bounds; this change adds
  paint clipping, not curved hit targets. `ContentMask::contains` can test both
  constraints explicitly.
- `ContentMask { bounds }` literals must migrate to `ContentMask::new(bounds)`
  (or specify the new fields). GPU records gain 32 bytes for original bounds and
  radii, rather than copying the mask through each shader varying.

## Known gaps

- The former native-WGSL `PolychromeSprite` 116/120-byte stride mismatch is fixed
  with explicit tail padding; layout tests check native struct spans and WebGL
  word strides. This does not substitute for Linux/web rendering verification.
- Rounded **path** masking is Metal-only; HLSL/WGSL paths retain rectangular
  clips. Metal uses antialiased mask coverage; HLSL/WGSL use a hard SDF discard.
- The WGSL/HLSL `pattern_dots` case and the Windows title bar changes were
  type-checked for `x86_64-pc-windows-msvc` but not shader-compiled (no
  fxc/dxc/naga run) or rendered on Windows; verify on Windows. `Surface`
  painting and the latency counters are macOS-specific.
- The rounded-mask change was compiled/rendered on macOS and WGSL was validated
  with Naga; Windows FXC compilation and Windows runtime rendering still require
  verification on Windows.

## Updating from upstream

1. Fetch `origin` and `upstream`; record their exact SHAs and merge-base. Keep
   the client's pinned `zed-fork` checkout untouched; work on a fork branch.
2. Merge the selected `upstream/main` commit into that branch. Resolve
   conflicts against the current crate layout (especially `gpui_apple`), not
   against pre-September file paths. Preserve the five fork-only changes or
   explicitly document their replacement, including the Windows parity commit.
3. Compare the merged tree with the selected upstream commit and re-verify
   every API, call site, crop/layout rule, scene field, shader path, and macOS
   counter above. Check Metal, WGSL, WebGL, and HLSL separately; do not infer
   parity from a successful macOS build. Check whether the known gaps remain.
4. Point a disposable Flowform client checkout at the candidate fork. Build
   the client against it and run the client tests; verify the image transform,
   crop, video-surface, dot-grid, and latency paths on their supported targets.
   Record the candidate and tested client revisions and results before updating
   the client's pin. Refresh this file against the final merged tree.

## Pending branches

- `flowform/bgra-surfaces-v2` at `d1bb0d7eb5` adds BGRA `CVPixelBuffer` surface
  support on top of `ccc83bcc04`. It is **not** an ancestor of fork `origin/main`;
  BGRA support is not part of the current base. Re-evaluate this section if
  that branch lands.
