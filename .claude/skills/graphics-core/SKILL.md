---
name: graphics-core
description: Framework-agnostic 2D/3D graphics fundamentals using Rust. Covers linear algebra (nalgebra/glam), coordinate systems, transforms (affine, perspective, projection), color spaces, paths/curves, compositing, and rasterization concepts. Use when implementing graphics math, coordinate transforms, color conversions, Bezier curves, or understanding rendering pipelines independent of any specific framework (iced, bevy, wgpu).
---

# Graphics Core

Framework-agnostic graphics programming fundamentals for Rust. This skill covers the mathematical and conceptual foundations that apply across all graphics frameworks.

## Linear Algebra with nalgebra

### Vectors

```rust
use nalgebra::{Vector2, Vector3, Vector4, Unit};

// 2D vectors
let v2 = Vector2::new(3.0_f32, 4.0);
let length = v2.magnitude();  // 5.0
let normalized = v2.normalize();  // (0.6, 0.8)

// 3D vectors
let v3 = Vector3::new(1.0, 2.0, 3.0);
let cross = v3.cross(&Vector3::new(0.0, 1.0, 0.0));
let dot = v3.dot(&Vector3::new(1.0, 0.0, 0.0));

// Unit vectors (guaranteed normalized)
let unit: Unit<Vector3<f32>> = Unit::new_normalize(v3);
```

### Matrices

```rust
use nalgebra::{Matrix2, Matrix3, Matrix4, Rotation2, Rotation3};

// 2D rotation matrix
let angle = std::f32::consts::FRAC_PI_4;  // 45 degrees
let rot2d = Rotation2::new(angle);
let rotated = rot2d * Vector2::new(1.0, 0.0);

// 3D rotation around axis
let axis = Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0));
let rot3d = Rotation3::from_axis_angle(&axis, angle);

// 4x4 transformation matrix
let transform = Matrix4::new_translation(&Vector3::new(10.0, 20.0, 0.0));
```

### Affine Transforms (2D)

```rust
use nalgebra::{Affine2, Translation2, Rotation2, Scale2};

// Compose: scale -> rotate -> translate
let scale = Scale2::new(2.0, 2.0);
let rotate = Rotation2::new(std::f32::consts::FRAC_PI_2);
let translate = Translation2::new(100.0, 50.0);

// Apply in order (right to left)
let transform = translate * rotate * scale;
let point = nalgebra::Point2::new(1.0, 0.0);
let transformed = transform * point;
```

### Projections (3D)

```rust
use nalgebra::{Perspective3, Orthographic3, Point3};

// Perspective projection (frustum)
let perspective = Perspective3::new(
    16.0 / 9.0,  // aspect ratio
    std::f32::consts::FRAC_PI_4,  // 45° FOV
    0.1,   // near plane
    100.0, // far plane
);
let proj_matrix = perspective.as_matrix();

// Orthographic projection
let ortho = Orthographic3::new(
    -10.0, 10.0,  // left, right
    -10.0, 10.0,  // bottom, top
    0.1, 100.0,   // near, far
);
```

### View Matrix (Camera)

```rust
use nalgebra::{Isometry3, Point3, Vector3};

// Look-at camera
let eye = Point3::new(0.0, 5.0, 10.0);
let target = Point3::new(0.0, 0.0, 0.0);
let up = Vector3::new(0.0, 1.0, 0.0);

let view = Isometry3::look_at_rh(&eye, &target, &up);
let view_matrix = view.to_homogeneous();
```

## Coordinate Systems

### Screen vs World Coordinates

```rust
/// Convert screen coordinates (pixels, origin top-left) to NDC
fn screen_to_ndc(screen: Vector2<f32>, viewport: Vector2<f32>) -> Vector2<f32> {
    Vector2::new(
        (2.0 * screen.x / viewport.x) - 1.0,
        1.0 - (2.0 * screen.y / viewport.y),  // Y flipped
    )
}

/// Convert NDC to screen coordinates
fn ndc_to_screen(ndc: Vector2<f32>, viewport: Vector2<f32>) -> Vector2<f32> {
    Vector2::new(
        (ndc.x + 1.0) * viewport.x / 2.0,
        (1.0 - ndc.y) * viewport.y / 2.0,
    )
}
```

### PDF Coordinate System

PDF uses bottom-left origin with Y increasing upward:

```rust
/// Convert PDF coordinates to screen coordinates
fn pdf_to_screen(
    pdf_point: Vector2<f32>,
    page_height: f32,
    scale: f32,
) -> Vector2<f32> {
    Vector2::new(
        pdf_point.x * scale,
        (page_height - pdf_point.y) * scale,  // Flip Y
    )
}

/// Apply PDF CTM (Current Transformation Matrix)
fn apply_ctm(point: Vector2<f32>, ctm: &[f32; 6]) -> Vector2<f32> {
    // CTM = [a b c d e f]
    // x' = a*x + c*y + e
    // y' = b*x + d*y + f
    Vector2::new(
        ctm[0] * point.x + ctm[2] * point.y + ctm[4],
        ctm[1] * point.x + ctm[3] * point.y + ctm[5],
    )
}
```

## Bezier Curves

### Quadratic Bezier

```rust
/// Evaluate quadratic Bezier at parameter t ∈ [0, 1]
fn quadratic_bezier(
    p0: Vector2<f32>,
    p1: Vector2<f32>,  // control point
    p2: Vector2<f32>,
    t: f32,
) -> Vector2<f32> {
    let one_minus_t = 1.0 - t;
    p0 * (one_minus_t * one_minus_t)
        + p1 * (2.0 * one_minus_t * t)
        + p2 * (t * t)
}
```

### Cubic Bezier

```rust
/// Evaluate cubic Bezier at parameter t ∈ [0, 1]
fn cubic_bezier(
    p0: Vector2<f32>,
    p1: Vector2<f32>,  // control point 1
    p2: Vector2<f32>,  // control point 2
    p3: Vector2<f32>,
    t: f32,
) -> Vector2<f32> {
    let one_minus_t = 1.0 - t;
    let t2 = t * t;
    let one_minus_t2 = one_minus_t * one_minus_t;

    p0 * (one_minus_t2 * one_minus_t)
        + p1 * (3.0 * one_minus_t2 * t)
        + p2 * (3.0 * one_minus_t * t2)
        + p3 * (t2 * t)
}

/// Approximate cubic Bezier with line segments
fn flatten_cubic_bezier(
    p0: Vector2<f32>,
    p1: Vector2<f32>,
    p2: Vector2<f32>,
    p3: Vector2<f32>,
    tolerance: f32,
) -> Vec<Vector2<f32>> {
    let mut points = vec![p0];
    flatten_recursive(p0, p1, p2, p3, tolerance, &mut points);
    points.push(p3);
    points
}

fn flatten_recursive(
    p0: Vector2<f32>,
    p1: Vector2<f32>,
    p2: Vector2<f32>,
    p3: Vector2<f32>,
    tolerance: f32,
    points: &mut Vec<Vector2<f32>>,
) {
    // Check if curve is flat enough
    let d1 = (p1 - p0).magnitude();
    let d2 = (p2 - p1).magnitude();
    let d3 = (p3 - p2).magnitude();
    let chord = (p3 - p0).magnitude();

    if (d1 + d2 + d3 - chord) < tolerance {
        return;
    }

    // De Casteljau subdivision at t=0.5
    let p01 = (p0 + p1) * 0.5;
    let p12 = (p1 + p2) * 0.5;
    let p23 = (p2 + p3) * 0.5;
    let p012 = (p01 + p12) * 0.5;
    let p123 = (p12 + p23) * 0.5;
    let mid = (p012 + p123) * 0.5;

    flatten_recursive(p0, p01, p012, mid, tolerance, points);
    points.push(mid);
    flatten_recursive(mid, p123, p23, p3, tolerance, points);
}
```

## Color Spaces

### sRGB ↔ Linear RGB

```rust
/// Convert sRGB component to linear (gamma decode)
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear component to sRGB (gamma encode)
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Full color conversion
fn srgb_to_linear_rgb(srgb: [f32; 3]) -> [f32; 3] {
    [
        srgb_to_linear(srgb[0]),
        srgb_to_linear(srgb[1]),
        srgb_to_linear(srgb[2]),
    ]
}
```

### HSV ↔ RGB

```rust
/// Convert HSV to RGB
/// h: 0-360, s: 0-1, v: 0-1
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());

    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let m = v - c;
    [r1 + m, g1 + m, b1 + m]
}
```

### Alpha Compositing (Porter-Duff)

```rust
/// Source-over compositing (most common)
fn composite_over(
    src: [f32; 4],  // RGBA, premultiplied alpha
    dst: [f32; 4],
) -> [f32; 4] {
    let out_a = src[3] + dst[3] * (1.0 - src[3]);
    if out_a == 0.0 {
        return [0.0; 4];
    }
    [
        src[0] + dst[0] * (1.0 - src[3]),
        src[1] + dst[1] * (1.0 - src[3]),
        src[2] + dst[2] * (1.0 - src[3]),
        out_a,
    ]
}

/// Premultiply alpha
fn premultiply(rgba: [f32; 4]) -> [f32; 4] {
    [
        rgba[0] * rgba[3],
        rgba[1] * rgba[3],
        rgba[2] * rgba[3],
        rgba[3],
    ]
}

/// Unpremultiply alpha
fn unpremultiply(rgba: [f32; 4]) -> [f32; 4] {
    if rgba[3] == 0.0 {
        return [0.0; 4];
    }
    [
        rgba[0] / rgba[3],
        rgba[1] / rgba[3],
        rgba[2] / rgba[3],
        rgba[3],
    ]
}
```

## Rasterization Concepts

### Scanline Fill (Even-Odd Rule)

```rust
/// Check if point is inside polygon using even-odd rule
fn point_in_polygon_even_odd(point: Vector2<f32>, polygon: &[Vector2<f32>]) -> bool {
    let mut inside = false;
    let n = polygon.len();

    for i in 0..n {
        let j = (i + 1) % n;
        let vi = polygon[i];
        let vj = polygon[j];

        if ((vi.y > point.y) != (vj.y > point.y))
            && (point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y) + vi.x)
        {
            inside = !inside;
        }
    }
    inside
}
```

### Line Clipping (Cohen-Sutherland)

```rust
const INSIDE: u8 = 0;
const LEFT: u8 = 1;
const RIGHT: u8 = 2;
const BOTTOM: u8 = 4;
const TOP: u8 = 8;

fn compute_outcode(x: f32, y: f32, xmin: f32, xmax: f32, ymin: f32, ymax: f32) -> u8 {
    let mut code = INSIDE;
    if x < xmin { code |= LEFT; }
    else if x > xmax { code |= RIGHT; }
    if y < ymin { code |= BOTTOM; }
    else if y > ymax { code |= TOP; }
    code
}
```

### Anti-Aliasing Basics

```rust
/// Simple box filter for coverage estimation
fn coverage_box_filter(
    pixel_center: Vector2<f32>,
    shape_sdf: impl Fn(Vector2<f32>) -> f32,  // signed distance
    samples: u32,
) -> f32 {
    let mut inside_count = 0u32;
    let sample_offset = 1.0 / (samples as f32 + 1.0);

    for sy in 0..samples {
        for sx in 0..samples {
            let sample = Vector2::new(
                pixel_center.x - 0.5 + sample_offset * (sx as f32 + 1.0),
                pixel_center.y - 0.5 + sample_offset * (sy as f32 + 1.0),
            );
            if shape_sdf(sample) <= 0.0 {
                inside_count += 1;
            }
        }
    }

    inside_count as f32 / (samples * samples) as f32
}
```

## Common Patterns

### Transform Stack

```rust
use nalgebra::Matrix3;

struct TransformStack {
    stack: Vec<Matrix3<f32>>,
    current: Matrix3<f32>,
}

impl TransformStack {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            current: Matrix3::identity(),
        }
    }

    fn push(&mut self) {
        self.stack.push(self.current);
    }

    fn pop(&mut self) -> bool {
        if let Some(m) = self.stack.pop() {
            self.current = m;
            true
        } else {
            false
        }
    }

    fn translate(&mut self, x: f32, y: f32) {
        let t = Matrix3::new_translation(&Vector2::new(x, y));
        self.current *= t;
    }

    fn rotate(&mut self, angle: f32) {
        let r = Matrix3::from_scaled_axis(Vector3::new(0.0, 0.0, angle));
        self.current *= r;
    }

    fn scale(&mut self, sx: f32, sy: f32) {
        self.current *= Matrix3::new_nonuniform_scaling(&Vector2::new(sx, sy));
    }
}
```

### Bounding Box

```rust
#[derive(Clone, Copy, Debug)]
struct BoundingBox {
    min: Vector2<f32>,
    max: Vector2<f32>,
}

impl BoundingBox {
    fn empty() -> Self {
        Self {
            min: Vector2::new(f32::INFINITY, f32::INFINITY),
            max: Vector2::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    fn include_point(&mut self, p: Vector2<f32>) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }

    fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    fn contains(&self, p: Vector2<f32>) -> bool {
        p.x >= self.min.x && p.x <= self.max.x
            && p.y >= self.min.y && p.y <= self.max.y
    }
}
```

## References

- `references/nalgebra_cheatsheet.md` - Complete nalgebra API patterns
- `references/color_theory.md` - Deep dive on color spaces and ICC profiles
- `references/pdf_graphics_model.md` - PDF-specific graphics state machine

## Quick Reference

| Operation | nalgebra Type | Common Use |
|-----------|---------------|------------|
| 2D point | `Point2<f32>` | Positions |
| 2D vector | `Vector2<f32>` | Directions, offsets |
| 2D transform | `Affine2<f32>` | Scale/rotate/translate |
| 3D point | `Point3<f32>` | 3D positions |
| 3D transform | `Isometry3<f32>` | Rigid body transform |
| Projection | `Matrix4<f32>` | View/projection |
| Unit vector | `Unit<Vector3<f32>>` | Normals, directions |
