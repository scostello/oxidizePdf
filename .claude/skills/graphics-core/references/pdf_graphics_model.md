# PDF Graphics Model

Graphics state machine and rendering model as defined in ISO 32000-1:2008. Essential reference for implementing PDF renderers and understanding how PDF content streams translate to pixels.

## Table of Contents

1. [Coordinate Systems](#coordinate-systems)
2. [Current Transformation Matrix (CTM)](#current-transformation-matrix-ctm)
3. [Graphics State](#graphics-state)
4. [Path Construction](#path-construction)
5. [Color Spaces](#color-spaces)
6. [Blend Modes](#blend-modes)
7. [Text Rendering](#text-rendering)
8. [Clipping](#clipping)
9. [Images](#images)

---

## Coordinate Systems

### User Space (Default)

PDF's default coordinate system:
- **Origin**: Bottom-left corner of the page
- **Units**: 1 unit = 1/72 inch (1 point)
- **Y-axis**: Increases upward
- **X-axis**: Increases rightward

```
        Y
        ↑
        │
        │    (page content)
        │
        └──────────────→ X
      (0,0)
```

### Device Space

The output device's coordinate system (screen, printer):
- **Origin**: Typically top-left
- **Units**: Device pixels
- **Y-axis**: Typically increases downward

### Conversion Formula

```rust
/// Convert PDF user space to device space
fn pdf_to_device(
    pdf_point: (f32, f32),
    page_height: f32,
    scale: f32,  // device pixels per point
) -> (f32, f32) {
    (
        pdf_point.0 * scale,
        (page_height - pdf_point.1) * scale,  // flip Y
    )
}
```

### Media Box vs Crop Box

```rust
/// Page boundaries (all in user space units)
struct PageBoxes {
    media_box: [f32; 4],  // Physical page size [x0, y0, x1, y1]
    crop_box: [f32; 4],   // Visible region (default = media_box)
    bleed_box: [f32; 4],  // Printing bleed area
    trim_box: [f32; 4],   // Intended final page size
    art_box: [f32; 4],    // Meaningful content extent
}
```

---

## Current Transformation Matrix (CTM)

The CTM transforms from user space to device space. It's a 3x3 matrix in homogeneous coordinates, but PDF represents it as 6 numbers:

```
┌         ┐     ┌           ┐
│ a  b  0 │     │ a  b  0   │
│ c  d  0 │  =  │ c  d  0   │
│ e  f  1 │     │ tx ty 1   │
└         ┘     └           ┘

PDF array: [a, b, c, d, e, f]
```

### Transform Application

```rust
/// Apply CTM to a point
fn transform_point(ctm: &[f32; 6], x: f32, y: f32) -> (f32, f32) {
    let [a, b, c, d, e, f] = *ctm;
    (
        a * x + c * y + e,
        b * x + d * y + f,
    )
}

/// Concatenate two transforms: result = m2 * m1
fn concat_ctm(m1: &[f32; 6], m2: &[f32; 6]) -> [f32; 6] {
    let [a1, b1, c1, d1, e1, f1] = *m1;
    let [a2, b2, c2, d2, e2, f2] = *m2;
    [
        a2 * a1 + b2 * c1,
        a2 * b1 + b2 * d1,
        c2 * a1 + d2 * c1,
        c2 * b1 + d2 * d1,
        e2 * a1 + f2 * c1 + e1,
        e2 * b1 + f2 * d1 + f1,
    ]
}
```

### Common Transforms

```rust
/// Identity matrix
const IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// Translation
fn translation(tx: f32, ty: f32) -> [f32; 6] {
    [1.0, 0.0, 0.0, 1.0, tx, ty]
}

/// Scaling
fn scaling(sx: f32, sy: f32) -> [f32; 6] {
    [sx, 0.0, 0.0, sy, 0.0, 0.0]
}

/// Rotation (angle in radians, counterclockwise)
fn rotation(angle: f32) -> [f32; 6] {
    let cos = angle.cos();
    let sin = angle.sin();
    [cos, sin, -sin, cos, 0.0, 0.0]
}

/// Skew
fn skew(alpha: f32, beta: f32) -> [f32; 6] {
    [1.0, alpha.tan(), beta.tan(), 1.0, 0.0, 0.0]
}
```

### PDF Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `cm` | Concatenate matrix to CTM | `1 0 0 1 100 200 cm` (translate) |
| `q` | Save graphics state (push) | |
| `Q` | Restore graphics state (pop) | |

---

## Graphics State

The graphics state contains all rendering parameters. It forms a stack with `q`/`Q` operators.

### State Parameters

```rust
#[derive(Clone)]
struct GraphicsState {
    // Transformation
    ctm: [f32; 6],

    // Color
    fill_color: Color,
    stroke_color: Color,
    fill_color_space: ColorSpace,
    stroke_color_space: ColorSpace,

    // Line style
    line_width: f32,
    line_cap: LineCap,      // 0=Butt, 1=Round, 2=Square
    line_join: LineJoin,    // 0=Miter, 1=Round, 2=Bevel
    miter_limit: f32,
    dash_pattern: DashPattern,

    // Text state
    text_state: TextState,

    // Transparency
    fill_alpha: f32,        // 0.0-1.0
    stroke_alpha: f32,
    blend_mode: BlendMode,

    // Clipping
    clipping_path: Option<Path>,

    // Misc
    flatness: f32,
    rendering_intent: RenderingIntent,
}

#[derive(Clone)]
struct TextState {
    font: Option<FontRef>,
    font_size: f32,
    char_spacing: f32,
    word_spacing: f32,
    horizontal_scaling: f32,  // percentage
    leading: f32,
    rise: f32,
    render_mode: TextRenderMode,
    text_matrix: [f32; 6],
    line_matrix: [f32; 6],
}
```

### State Operators

| Operator | Parameter | Description |
|----------|-----------|-------------|
| `w` | lineWidth | Set line width |
| `J` | lineCap | Set line cap style |
| `j` | lineJoin | Set line join style |
| `M` | miterLimit | Set miter limit |
| `d` | dashArray dashPhase | Set dash pattern |
| `ri` | intent | Set rendering intent |
| `i` | flatness | Set flatness tolerance |
| `gs` | dictName | Set from graphics state dict |

---

## Path Construction

Paths are built incrementally then painted (stroked/filled).

### Path Operators

| Operator | Args | Description |
|----------|------|-------------|
| `m` | x y | Move to (start subpath) |
| `l` | x y | Line to |
| `c` | x1 y1 x2 y2 x3 y3 | Cubic Bezier |
| `v` | x2 y2 x3 y3 | Cubic Bezier (cp1 = current point) |
| `y` | x1 y1 x3 y3 | Cubic Bezier (cp2 = endpoint) |
| `h` | | Close subpath |
| `re` | x y w h | Rectangle |

### Painting Operators

| Operator | Description |
|----------|-------------|
| `S` | Stroke path |
| `s` | Close and stroke |
| `f` | Fill (non-zero winding) |
| `F` | Fill (same as `f`) |
| `f*` | Fill (even-odd rule) |
| `B` | Fill and stroke (non-zero) |
| `B*` | Fill and stroke (even-odd) |
| `b` | Close, fill, stroke (non-zero) |
| `b*` | Close, fill, stroke (even-odd) |
| `n` | End path (no paint) |

### Path Construction in Rust

```rust
#[derive(Clone)]
enum PathSegment {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    CurveTo(f32, f32, f32, f32, f32, f32),  // cubic bezier
    ClosePath,
}

struct PathBuilder {
    segments: Vec<PathSegment>,
    current_point: Option<(f32, f32)>,
    start_point: Option<(f32, f32)>,
}

impl PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.segments.push(PathSegment::MoveTo(x, y));
        self.current_point = Some((x, y));
        self.start_point = Some((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.segments.push(PathSegment::LineTo(x, y));
        self.current_point = Some((x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.segments.push(PathSegment::CurveTo(x1, y1, x2, y2, x3, y3));
        self.current_point = Some((x3, y3));
    }

    fn close(&mut self) {
        self.segments.push(PathSegment::ClosePath);
        self.current_point = self.start_point;
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.move_to(x, y);
        self.line_to(x + w, y);
        self.line_to(x + w, y + h);
        self.line_to(x, y + h);
        self.close();
    }
}
```

### Fill Rules

**Non-zero winding rule** (default `f`):
- Count crossings: +1 for clockwise, -1 for counterclockwise
- Fill if count ≠ 0

**Even-odd rule** (`f*`):
- Count total crossings
- Fill if count is odd

```rust
fn point_in_path_nonzero(point: (f32, f32), path: &[PathSegment]) -> bool {
    let mut winding = 0i32;
    // ... count signed crossings
    winding != 0
}

fn point_in_path_evenodd(point: (f32, f32), path: &[PathSegment]) -> bool {
    let mut crossings = 0u32;
    // ... count crossings
    crossings % 2 == 1
}
```

---

## Color Spaces

### Device Color Spaces

```rust
enum DeviceColorSpace {
    DeviceGray,   // 1 component: gray (0=black, 1=white)
    DeviceRGB,    // 3 components: R, G, B (0-1 each)
    DeviceCMYK,   // 4 components: C, M, Y, K (0-1 each)
}
```

### Color Operators

| Operator | Description |
|----------|-------------|
| `g` | Set gray fill (0-1) |
| `G` | Set gray stroke |
| `rg` | Set RGB fill |
| `RG` | Set RGB stroke |
| `k` | Set CMYK fill |
| `K` | Set CMYK stroke |
| `cs` | Set fill color space |
| `CS` | Set stroke color space |
| `sc` / `scn` | Set fill color |
| `SC` / `SCN` | Set stroke color |

### Color Conversion

```rust
/// CMYK to RGB (simple conversion)
fn cmyk_to_rgb(c: f32, m: f32, y: f32, k: f32) -> (f32, f32, f32) {
    let r = (1.0 - c) * (1.0 - k);
    let g = (1.0 - m) * (1.0 - k);
    let b = (1.0 - y) * (1.0 - k);
    (r, g, b)
}

/// Gray to RGB
fn gray_to_rgb(gray: f32) -> (f32, f32, f32) {
    (gray, gray, gray)
}
```

---

## Blend Modes

PDF supports Porter-Duff compositing with additional blend modes.

### Separable Blend Modes

Applied per-component:

```rust
fn blend_normal(cb: f32, cs: f32) -> f32 { cs }
fn blend_multiply(cb: f32, cs: f32) -> f32 { cb * cs }
fn blend_screen(cb: f32, cs: f32) -> f32 { 1.0 - (1.0 - cb) * (1.0 - cs) }
fn blend_overlay(cb: f32, cs: f32) -> f32 {
    if cb <= 0.5 {
        2.0 * cb * cs
    } else {
        1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
    }
}
fn blend_darken(cb: f32, cs: f32) -> f32 { cb.min(cs) }
fn blend_lighten(cb: f32, cs: f32) -> f32 { cb.max(cs) }
fn blend_color_dodge(cb: f32, cs: f32) -> f32 {
    if cs >= 1.0 { 1.0 }
    else { (cb / (1.0 - cs)).min(1.0) }
}
fn blend_color_burn(cb: f32, cs: f32) -> f32 {
    if cs <= 0.0 { 0.0 }
    else { 1.0 - ((1.0 - cb) / cs).min(1.0) }
}
fn blend_hard_light(cb: f32, cs: f32) -> f32 { blend_overlay(cs, cb) }
fn blend_soft_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
    } else {
        let d = if cb <= 0.25 {
            ((16.0 * cb - 12.0) * cb + 4.0) * cb
        } else {
            cb.sqrt()
        };
        cb + (2.0 * cs - 1.0) * (d - cb)
    }
}
fn blend_difference(cb: f32, cs: f32) -> f32 { (cb - cs).abs() }
fn blend_exclusion(cb: f32, cs: f32) -> f32 { cb + cs - 2.0 * cb * cs }
```

### Alpha Compositing

```rust
/// Source-over with alpha
fn composite_over(
    src_rgb: (f32, f32, f32), src_a: f32,
    dst_rgb: (f32, f32, f32), dst_a: f32,
    blend_fn: fn(f32, f32) -> f32,
) -> ((f32, f32, f32), f32) {
    let out_a = src_a + dst_a * (1.0 - src_a);
    if out_a == 0.0 {
        return ((0.0, 0.0, 0.0), 0.0);
    }

    let blend_r = blend_fn(dst_rgb.0, src_rgb.0);
    let blend_g = blend_fn(dst_rgb.1, src_rgb.1);
    let blend_b = blend_fn(dst_rgb.2, src_rgb.2);

    let out_r = (src_a * (1.0 - dst_a) * src_rgb.0
               + src_a * dst_a * blend_r
               + (1.0 - src_a) * dst_a * dst_rgb.0) / out_a;
    let out_g = (src_a * (1.0 - dst_a) * src_rgb.1
               + src_a * dst_a * blend_g
               + (1.0 - src_a) * dst_a * dst_rgb.1) / out_a;
    let out_b = (src_a * (1.0 - dst_a) * src_rgb.2
               + src_a * dst_a * blend_b
               + (1.0 - src_a) * dst_a * dst_rgb.2) / out_a;

    ((out_r, out_g, out_b), out_a)
}
```

---

## Text Rendering

### Text State Operators

| Operator | Description |
|----------|-------------|
| `Tf` | Set font and size |
| `Tc` | Set character spacing |
| `Tw` | Set word spacing |
| `Tz` | Set horizontal scaling |
| `TL` | Set leading |
| `Ts` | Set text rise |
| `Tr` | Set text render mode |

### Text Positioning

| Operator | Description |
|----------|-------------|
| `BT` | Begin text object |
| `ET` | End text object |
| `Td` | Move to next line (offset) |
| `TD` | Move and set leading |
| `Tm` | Set text matrix |
| `T*` | Move to next line |

### Text Showing

| Operator | Description |
|----------|-------------|
| `Tj` | Show string |
| `TJ` | Show with positioning |
| `'` | Move to next line and show |
| `"` | Set spacing, move, and show |

### Text Rendering Modes

```rust
enum TextRenderMode {
    Fill = 0,           // Fill text
    Stroke = 1,         // Stroke text
    FillStroke = 2,     // Fill then stroke
    Invisible = 3,      // Neither (for clipping)
    FillClip = 4,       // Fill and add to clipping
    StrokeClip = 5,     // Stroke and add to clipping
    FillStrokeClip = 6, // Fill, stroke, and clip
    Clip = 7,           // Add to clipping only
}
```

### Text Matrix Calculation

```rust
/// Calculate glyph position in user space
fn glyph_position(
    text_matrix: &[f32; 6],
    ctm: &[f32; 6],
) -> [f32; 6] {
    concat_ctm(text_matrix, ctm)
}

/// Advance text matrix after glyph
fn advance_text_matrix(
    tm: &mut [f32; 6],
    glyph_width: f32,
    font_size: f32,
    char_spacing: f32,
    word_spacing: f32,
    horizontal_scaling: f32,
    is_space: bool,
) {
    let tx = (glyph_width * font_size + char_spacing
             + if is_space { word_spacing } else { 0.0 })
             * horizontal_scaling / 100.0;

    // Translate text matrix
    tm[4] += tx * tm[0];
    tm[5] += tx * tm[1];
}
```

---

## Clipping

Clipping restricts painting to a region.

### Clipping Operators

| Operator | Description |
|----------|-------------|
| `W` | Set clipping path (non-zero) |
| `W*` | Set clipping path (even-odd) |

Clipping paths intersect with existing clip:

```rust
fn intersect_clip(current: &Path, new: &Path) -> Path {
    // Result is intersection of both regions
    // Implementation depends on path representation
}
```

**Important**: Clipping paths can only be reduced (intersected), never expanded. Use `q`/`Q` to restore previous clip.

---

## Images

### Image XObjects

```rust
struct ImageXObject {
    width: u32,
    height: u32,
    color_space: ColorSpace,
    bits_per_component: u8,  // 1, 2, 4, 8, or 16
    data: Vec<u8>,           // Decoded pixel data
    mask: Option<ImageMask>,
    soft_mask: Option<SoftMask>,
    decode: Option<Vec<f32>>, // Decode array
    interpolate: bool,
}
```

### Image Placement

Images are placed via the CTM:

```rust
// Image draws into unit square (0,0)-(1,1)
// CTM scales/positions to final location
let ctm = [
    width, 0.0,    // scale X
    0.0, height,   // scale Y
    x, y,          // position
];
// Then: Do name (invoke XObject)
```

### Decode Array

Maps raw sample values to color component values:

```rust
fn decode_sample(
    sample: u8,
    bits: u8,
    decode_min: f32,
    decode_max: f32,
) -> f32 {
    let max_sample = (1 << bits) - 1;
    let normalized = sample as f32 / max_sample as f32;
    decode_min + normalized * (decode_max - decode_min)
}
```

---

## Quick Reference: Operator Categories

| Category | Operators |
|----------|-----------|
| Graphics state | `q Q cm w J j M d ri i gs` |
| Path construction | `m l c v y h re` |
| Path painting | `S s f F f* B B* b b* n` |
| Clipping | `W W*` |
| Color | `g G rg RG k K cs CS sc SC scn SCN` |
| Text state | `Tc Tw Tz TL Tf Tr Ts` |
| Text positioning | `BT ET Td TD Tm T*` |
| Text showing | `Tj TJ ' "` |
| XObjects | `Do` |
| Marked content | `MP DP BMC BDC EMC` |
| Compatibility | `BX EX` |
