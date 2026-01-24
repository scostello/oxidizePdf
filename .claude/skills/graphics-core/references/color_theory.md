# Color Theory for Graphics Programming

Comprehensive reference for color spaces, conversions, and color management in Rust graphics applications.

## Table of Contents

1. [Color Models](#color-models)
2. [Gamma and Transfer Functions](#gamma-and-transfer-functions)
3. [Color Space Conversions](#color-space-conversions)
4. [Perceptual Color Spaces](#perceptual-color-spaces)
5. [ICC Color Management](#icc-color-management)
6. [PDF Color Spaces](#pdf-color-spaces)
7. [Alpha and Premultiplication](#alpha-and-premultiplication)
8. [Practical Patterns](#practical-patterns)

---

## Color Models

### RGB (Red, Green, Blue)

Additive color model for displays.

```rust
#[derive(Clone, Copy, Debug)]
struct Rgb {
    r: f32,  // 0.0 - 1.0
    g: f32,
    b: f32,
}

impl Rgb {
    fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    fn to_u8(&self) -> (u8, u8, u8) {
        (
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }

    fn luminance(&self) -> f32 {
        // Rec. 709 coefficients (assumes linear RGB)
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
}
```

### CMYK (Cyan, Magenta, Yellow, Key/Black)

Subtractive color model for printing.

```rust
#[derive(Clone, Copy, Debug)]
struct Cmyk {
    c: f32,  // 0.0 - 1.0
    m: f32,
    y: f32,
    k: f32,
}

impl Cmyk {
    /// Simple conversion to RGB (no ICC profile)
    fn to_rgb_naive(&self) -> Rgb {
        Rgb {
            r: (1.0 - self.c) * (1.0 - self.k),
            g: (1.0 - self.m) * (1.0 - self.k),
            b: (1.0 - self.y) * (1.0 - self.k),
        }
    }

    /// Simple conversion from RGB
    fn from_rgb_naive(rgb: &Rgb) -> Self {
        let k = 1.0 - rgb.r.max(rgb.g).max(rgb.b);
        if k >= 1.0 {
            return Cmyk { c: 0.0, m: 0.0, y: 0.0, k: 1.0 };
        }
        Cmyk {
            c: (1.0 - rgb.r - k) / (1.0 - k),
            m: (1.0 - rgb.g - k) / (1.0 - k),
            y: (1.0 - rgb.b - k) / (1.0 - k),
            k,
        }
    }
}
```

**Important**: Real CMYK↔RGB conversion requires ICC profiles. The naive formulas above are only approximations.

### HSV/HSB (Hue, Saturation, Value/Brightness)

Intuitive for color picking and adjustment.

```rust
#[derive(Clone, Copy, Debug)]
struct Hsv {
    h: f32,  // 0.0 - 360.0 (degrees)
    s: f32,  // 0.0 - 1.0
    v: f32,  // 0.0 - 1.0
}

impl Hsv {
    fn to_rgb(&self) -> Rgb {
        if self.s == 0.0 {
            return Rgb::new(self.v, self.v, self.v);
        }

        let h = self.h / 60.0;
        let i = h.floor() as i32;
        let f = h - i as f32;

        let p = self.v * (1.0 - self.s);
        let q = self.v * (1.0 - self.s * f);
        let t = self.v * (1.0 - self.s * (1.0 - f));

        match i % 6 {
            0 => Rgb::new(self.v, t, p),
            1 => Rgb::new(q, self.v, p),
            2 => Rgb::new(p, self.v, t),
            3 => Rgb::new(p, q, self.v),
            4 => Rgb::new(t, p, self.v),
            _ => Rgb::new(self.v, p, q),
        }
    }

    fn from_rgb(rgb: &Rgb) -> Self {
        let max = rgb.r.max(rgb.g).max(rgb.b);
        let min = rgb.r.min(rgb.g).min(rgb.b);
        let delta = max - min;

        let v = max;
        let s = if max == 0.0 { 0.0 } else { delta / max };

        let h = if delta == 0.0 {
            0.0
        } else if max == rgb.r {
            60.0 * (((rgb.g - rgb.b) / delta) % 6.0)
        } else if max == rgb.g {
            60.0 * ((rgb.b - rgb.r) / delta + 2.0)
        } else {
            60.0 * ((rgb.r - rgb.g) / delta + 4.0)
        };

        Hsv {
            h: if h < 0.0 { h + 360.0 } else { h },
            s,
            v,
        }
    }
}
```

### HSL (Hue, Saturation, Lightness)

Similar to HSV but with different saturation/lightness model.

```rust
#[derive(Clone, Copy, Debug)]
struct Hsl {
    h: f32,  // 0.0 - 360.0
    s: f32,  // 0.0 - 1.0
    l: f32,  // 0.0 - 1.0
}

impl Hsl {
    fn to_rgb(&self) -> Rgb {
        if self.s == 0.0 {
            return Rgb::new(self.l, self.l, self.l);
        }

        let c = (1.0 - (2.0 * self.l - 1.0).abs()) * self.s;
        let x = c * (1.0 - ((self.h / 60.0) % 2.0 - 1.0).abs());
        let m = self.l - c / 2.0;

        let (r1, g1, b1) = match (self.h / 60.0) as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Rgb::new(r1 + m, g1 + m, b1 + m)
    }

    fn from_rgb(rgb: &Rgb) -> Self {
        let max = rgb.r.max(rgb.g).max(rgb.b);
        let min = rgb.r.min(rgb.g).min(rgb.b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        let h = if delta == 0.0 {
            0.0
        } else if max == rgb.r {
            60.0 * (((rgb.g - rgb.b) / delta) % 6.0)
        } else if max == rgb.g {
            60.0 * ((rgb.b - rgb.r) / delta + 2.0)
        } else {
            60.0 * ((rgb.r - rgb.g) / delta + 4.0)
        };

        Hsl {
            h: if h < 0.0 { h + 360.0 } else { h },
            s,
            l,
        }
    }
}
```

---

## Gamma and Transfer Functions

### The Problem

Human vision perceives brightness non-linearly. A pixel value of 128 doesn't appear half as bright as 255.

### sRGB Transfer Function

The standard for web and most displays.

```rust
/// sRGB gamma: encode linear → sRGB (for display)
fn linear_to_srgb(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

/// sRGB inverse gamma: decode sRGB → linear (for computation)
fn srgb_to_linear(srgb: f32) -> f32 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

/// Full RGB conversion
fn rgb_srgb_to_linear(srgb: &Rgb) -> Rgb {
    Rgb {
        r: srgb_to_linear(srgb.r),
        g: srgb_to_linear(srgb.g),
        b: srgb_to_linear(srgb.b),
    }
}

fn rgb_linear_to_srgb(linear: &Rgb) -> Rgb {
    Rgb {
        r: linear_to_srgb(linear.r),
        g: linear_to_srgb(linear.g),
        b: linear_to_srgb(linear.b),
    }
}
```

### Simple Gamma (Power Function)

Approximation used in some contexts:

```rust
const GAMMA: f32 = 2.2;

fn gamma_encode(linear: f32) -> f32 {
    linear.powf(1.0 / GAMMA)
}

fn gamma_decode(encoded: f32) -> f32 {
    encoded.powf(GAMMA)
}
```

### When to Use Linear vs Gamma

| Operation | Use Linear RGB |
|-----------|----------------|
| Blending/compositing | ✓ Yes |
| Interpolation | ✓ Yes |
| Lighting calculations | ✓ Yes |
| Physical simulations | ✓ Yes |
| Display output | ✗ Use sRGB |
| Image file storage | ✗ Usually sRGB |
| Color picking UI | ✗ Usually sRGB |

**Rule**: Do math in linear space, store/display in gamma space.

---

## Color Space Conversions

### RGB ↔ XYZ (CIE 1931)

XYZ is a device-independent color space.

```rust
/// sRGB to XYZ (D65 illuminant)
fn srgb_to_xyz(rgb: &Rgb) -> (f32, f32, f32) {
    // First convert to linear RGB
    let r = srgb_to_linear(rgb.r);
    let g = srgb_to_linear(rgb.g);
    let b = srgb_to_linear(rgb.b);

    // sRGB to XYZ matrix (D65)
    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;

    (x, y, z)
}

/// XYZ to sRGB (D65 illuminant)
fn xyz_to_srgb(x: f32, y: f32, z: f32) -> Rgb {
    // XYZ to linear sRGB matrix
    let r =  3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
    let g = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
    let b =  0.0556434 * x - 0.2040259 * y + 1.0572252 * z;

    // Convert to sRGB
    Rgb {
        r: linear_to_srgb(r),
        g: linear_to_srgb(g),
        b: linear_to_srgb(b),
    }
}
```

### XYZ ↔ Lab (CIELAB)

Perceptually uniform color space.

```rust
/// D65 reference white
const D65_XN: f32 = 0.95047;
const D65_YN: f32 = 1.00000;
const D65_ZN: f32 = 1.08883;

fn lab_f(t: f32) -> f32 {
    const DELTA: f32 = 6.0 / 29.0;
    const DELTA_CUBED: f32 = DELTA * DELTA * DELTA;

    if t > DELTA_CUBED {
        t.cbrt()
    } else {
        t / (3.0 * DELTA * DELTA) + 4.0 / 29.0
    }
}

fn lab_f_inv(t: f32) -> f32 {
    const DELTA: f32 = 6.0 / 29.0;

    if t > DELTA {
        t * t * t
    } else {
        3.0 * DELTA * DELTA * (t - 4.0 / 29.0)
    }
}

#[derive(Clone, Copy, Debug)]
struct Lab {
    l: f32,  // 0 - 100
    a: f32,  // ~-128 to 128 (green to red)
    b: f32,  // ~-128 to 128 (blue to yellow)
}

impl Lab {
    fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        let fx = lab_f(x / D65_XN);
        let fy = lab_f(y / D65_YN);
        let fz = lab_f(z / D65_ZN);

        Lab {
            l: 116.0 * fy - 16.0,
            a: 500.0 * (fx - fy),
            b: 200.0 * (fy - fz),
        }
    }

    fn to_xyz(&self) -> (f32, f32, f32) {
        let fy = (self.l + 16.0) / 116.0;
        let fx = self.a / 500.0 + fy;
        let fz = fy - self.b / 200.0;

        (
            D65_XN * lab_f_inv(fx),
            D65_YN * lab_f_inv(fy),
            D65_ZN * lab_f_inv(fz),
        )
    }

    fn from_srgb(rgb: &Rgb) -> Self {
        let (x, y, z) = srgb_to_xyz(rgb);
        Lab::from_xyz(x, y, z)
    }

    fn to_srgb(&self) -> Rgb {
        let (x, y, z) = self.to_xyz();
        xyz_to_srgb(x, y, z)
    }
}
```

---

## Perceptual Color Spaces

### Color Difference (Delta E)

Measure perceptual difference between colors.

```rust
impl Lab {
    /// CIE76 Delta E (simple Euclidean)
    fn delta_e_76(&self, other: &Lab) -> f32 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        (dl * dl + da * da + db * db).sqrt()
    }

    /// CIE94 Delta E (improved, asymmetric)
    fn delta_e_94(&self, other: &Lab) -> f32 {
        let dl = self.l - other.l;
        let c1 = (self.a * self.a + self.b * self.b).sqrt();
        let c2 = (other.a * other.a + other.b * other.b).sqrt();
        let dc = c1 - c2;
        let da = self.a - other.a;
        let db = self.b - other.b;
        let dh_sq = da * da + db * db - dc * dc;
        let dh = if dh_sq > 0.0 { dh_sq.sqrt() } else { 0.0 };

        // Graphic arts constants
        let kl = 1.0;
        let k1 = 0.045;
        let k2 = 0.015;

        let sl = 1.0;
        let sc = 1.0 + k1 * c1;
        let sh = 1.0 + k2 * c1;

        let term1 = dl / (kl * sl);
        let term2 = dc / sc;
        let term3 = dh / sh;

        (term1 * term1 + term2 * term2 + term3 * term3).sqrt()
    }
}

/// Perceptual difference thresholds
/// < 1.0: Not perceptible
/// 1-2: Perceptible through close observation
/// 2-10: Perceptible at a glance
/// 11-49: Colors are more similar than opposite
/// 100: Colors are exact opposite
```

### Oklab

Modern perceptually uniform color space (2020).

```rust
#[derive(Clone, Copy, Debug)]
struct Oklab {
    l: f32,  // 0 - 1
    a: f32,  // ~-0.4 to 0.4
    b: f32,  // ~-0.4 to 0.4
}

impl Oklab {
    fn from_linear_rgb(rgb: &Rgb) -> Self {
        let l = 0.4122214708 * rgb.r + 0.5363325363 * rgb.g + 0.0514459929 * rgb.b;
        let m = 0.2119034982 * rgb.r + 0.6806995451 * rgb.g + 0.1073969566 * rgb.b;
        let s = 0.0883024619 * rgb.r + 0.2817188376 * rgb.g + 0.6299787005 * rgb.b;

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        Oklab {
            l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
        }
    }

    fn to_linear_rgb(&self) -> Rgb {
        let l_ = self.l + 0.3963377774 * self.a + 0.2158037573 * self.b;
        let m_ = self.l - 0.1055613458 * self.a - 0.0638541728 * self.b;
        let s_ = self.l - 0.0894841775 * self.a - 1.2914855480 * self.b;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        Rgb {
            r:  4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            g: -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            b: -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
        }
    }
}
```

**Advantages of Oklab**:
- Better perceptual uniformity than Lab
- Hue linearity (interpolation doesn't shift hue)
- Predictable lightness

---

## ICC Color Management

### Profile Structure

```rust
/// Simplified ICC profile representation
struct IccProfile {
    /// Profile class
    class: ProfileClass,
    /// Color space of data
    color_space: ColorSpaceSignature,
    /// Profile connection space (XYZ or Lab)
    pcs: ConnectionSpace,
    /// Rendering intent
    intent: RenderingIntent,
    /// Transform tables
    transforms: ProfileTransforms,
}

enum ProfileClass {
    Input,      // Scanner, camera
    Display,    // Monitor
    Output,     // Printer
    DeviceLink, // Device-to-device
    ColorSpace, // Color space conversion
    Abstract,   // Effects
    NamedColor, // Spot colors
}

enum RenderingIntent {
    Perceptual,           // Preserve relationships, compress gamut
    RelativeColorimetric, // Match colors, clip out-of-gamut
    Saturation,           // Maximize saturation
    AbsoluteColorimetric, // Match exactly, including white point
}
```

### Profile Application

```rust
/// Color management workflow
fn convert_with_profiles(
    src_color: &Rgb,
    src_profile: &IccProfile,
    dst_profile: &IccProfile,
    intent: RenderingIntent,
) -> Rgb {
    // 1. Convert source color to PCS (Profile Connection Space)
    let pcs = src_profile.to_pcs(src_color, intent);

    // 2. Convert PCS to destination color
    dst_profile.from_pcs(&pcs, intent)
}

/// Simplified transform (real ICC uses lookup tables)
impl IccProfile {
    fn to_pcs(&self, color: &Rgb, intent: RenderingIntent) -> Xyz {
        // Apply profile's A2B (device to PCS) transform
        // Real implementation uses multi-dimensional LUTs
        unimplemented!()
    }

    fn from_pcs(&self, pcs: &Xyz, intent: RenderingIntent) -> Rgb {
        // Apply profile's B2A (PCS to device) transform
        unimplemented!()
    }
}
```

### Common ICC Profiles

| Profile | Description |
|---------|-------------|
| sRGB IEC61966-2.1 | Standard web/display |
| Adobe RGB (1998) | Wider gamut for photography |
| ProPhoto RGB | Very wide gamut |
| Display P3 | Apple displays, HDR |
| FOGRA39 | European CMYK printing |
| SWOP | US web offset printing |

---

## PDF Color Spaces

### Device Color Spaces

```rust
enum PdfDeviceColorSpace {
    /// Single component: 0 (black) to 1 (white)
    DeviceGray,
    /// Three components: R, G, B (0-1 each)
    DeviceRGB,
    /// Four components: C, M, Y, K (0-1 each)
    DeviceCMYK,
}
```

### CIE-Based Color Spaces

```rust
enum PdfCieColorSpace {
    /// CIE XYZ
    CalGray {
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        gamma: Option<f32>,
    },
    CalRGB {
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        gamma: Option<[f32; 3]>,
        matrix: Option<[f32; 9]>,
    },
    Lab {
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        range: Option<[f32; 4]>,  // [a_min, a_max, b_min, b_max]
    },
    ICCBased {
        profile: Vec<u8>,  // Embedded ICC profile
        alternate: Option<Box<PdfColorSpace>>,
    },
}
```

### Special Color Spaces

```rust
enum PdfSpecialColorSpace {
    /// Indexed/palette color
    Indexed {
        base: Box<PdfColorSpace>,
        hival: u8,  // 0-255
        lookup: Vec<u8>,  // Color table
    },
    /// Separation (spot colors)
    Separation {
        name: String,
        alternate: Box<PdfColorSpace>,
        tint_transform: Function,
    },
    /// DeviceN (multiple separations)
    DeviceN {
        names: Vec<String>,
        alternate: Box<PdfColorSpace>,
        tint_transform: Function,
        attributes: Option<DeviceNAttributes>,
    },
    /// Pattern color space
    Pattern {
        underlying: Option<Box<PdfColorSpace>>,
    },
}
```

### Color Space Resolution

```rust
/// Resolve PDF color to device RGB for rendering
fn resolve_pdf_color(
    color: &[f32],
    color_space: &PdfColorSpace,
    default_cmyk_profile: Option<&IccProfile>,
) -> Rgb {
    match color_space {
        PdfColorSpace::DeviceGray => {
            let g = color[0];
            Rgb::new(g, g, g)
        }
        PdfColorSpace::DeviceRGB => {
            Rgb::new(color[0], color[1], color[2])
        }
        PdfColorSpace::DeviceCMYK => {
            // Use profile if available, else naive conversion
            match default_cmyk_profile {
                Some(profile) => convert_with_profile(color, profile),
                None => Cmyk {
                    c: color[0], m: color[1],
                    y: color[2], k: color[3],
                }.to_rgb_naive(),
            }
        }
        PdfColorSpace::CalRGB { gamma, matrix, .. } => {
            // Apply gamma, then matrix transform to XYZ, then to sRGB
            unimplemented!()
        }
        PdfColorSpace::ICCBased { profile, .. } => {
            // Use embedded ICC profile
            unimplemented!()
        }
        PdfColorSpace::Indexed { base, lookup, .. } => {
            let index = color[0] as usize;
            let components = lookup_indexed_color(index, base, lookup);
            resolve_pdf_color(&components, base, default_cmyk_profile)
        }
        _ => unimplemented!(),
    }
}
```

---

## Alpha and Premultiplication

### Straight vs Premultiplied Alpha

```rust
/// Straight alpha: color values are independent of alpha
struct StraightRgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

/// Premultiplied alpha: color values are multiplied by alpha
/// r_premul = r * a, etc.
struct PremultipliedRgba {
    r: f32,  // Actually r * a
    g: f32,  // Actually g * a
    b: f32,  // Actually b * a
    a: f32,
}

impl StraightRgba {
    fn to_premultiplied(&self) -> PremultipliedRgba {
        PremultipliedRgba {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }
}

impl PremultipliedRgba {
    fn to_straight(&self) -> StraightRgba {
        if self.a == 0.0 {
            StraightRgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
        } else {
            StraightRgba {
                r: self.r / self.a,
                g: self.g / self.a,
                b: self.b / self.a,
                a: self.a,
            }
        }
    }
}
```

### Why Premultiplied?

1. **Faster compositing**: No division needed
2. **Correct interpolation**: Avoids color bleeding at edges
3. **Simpler blending**: Porter-Duff operations work directly

```rust
/// Source-over compositing (premultiplied)
fn composite_over_premul(
    src: &PremultipliedRgba,
    dst: &PremultipliedRgba,
) -> PremultipliedRgba {
    let inv_src_a = 1.0 - src.a;
    PremultipliedRgba {
        r: src.r + dst.r * inv_src_a,
        g: src.g + dst.g * inv_src_a,
        b: src.b + dst.b * inv_src_a,
        a: src.a + dst.a * inv_src_a,
    }
}
```

---

## Practical Patterns

### Color Interpolation

```rust
/// Interpolate in linear RGB space (correct)
fn lerp_colors_correct(c1: &Rgb, c2: &Rgb, t: f32) -> Rgb {
    // Convert to linear
    let l1 = rgb_srgb_to_linear(c1);
    let l2 = rgb_srgb_to_linear(c2);

    // Interpolate in linear space
    let lerped = Rgb {
        r: l1.r + (l2.r - l1.r) * t,
        g: l1.g + (l2.g - l1.g) * t,
        b: l1.b + (l2.b - l1.b) * t,
    };

    // Convert back to sRGB
    rgb_linear_to_srgb(&lerped)
}

/// Interpolate in Oklab (better for hue)
fn lerp_colors_oklab(c1: &Rgb, c2: &Rgb, t: f32) -> Rgb {
    let l1 = rgb_srgb_to_linear(c1);
    let l2 = rgb_srgb_to_linear(c2);

    let ok1 = Oklab::from_linear_rgb(&l1);
    let ok2 = Oklab::from_linear_rgb(&l2);

    let lerped = Oklab {
        l: ok1.l + (ok2.l - ok1.l) * t,
        a: ok1.a + (ok2.a - ok1.a) * t,
        b: ok1.b + (ok2.b - ok1.b) * t,
    };

    let linear = lerped.to_linear_rgb();
    rgb_linear_to_srgb(&linear)
}
```

### Gamut Mapping

When colors are out of destination gamut:

```rust
/// Simple gamut clipping (fast but can shift hue)
fn clip_to_gamut(rgb: &Rgb) -> Rgb {
    Rgb {
        r: rgb.r.clamp(0.0, 1.0),
        g: rgb.g.clamp(0.0, 1.0),
        b: rgb.b.clamp(0.0, 1.0),
    }
}

/// Preserve lightness gamut mapping (better quality)
fn map_to_gamut_preserve_l(rgb: &Rgb) -> Rgb {
    let lab = Lab::from_srgb(rgb);

    // Binary search to find in-gamut color with same L
    let mut low_chroma = 0.0;
    let mut high_chroma = (lab.a * lab.a + lab.b * lab.b).sqrt();

    for _ in 0..16 {
        let mid = (low_chroma + high_chroma) / 2.0;
        let scale = if high_chroma > 0.0 { mid / high_chroma } else { 0.0 };

        let test_lab = Lab {
            l: lab.l,
            a: lab.a * scale,
            b: lab.b * scale,
        };
        let test_rgb = test_lab.to_srgb();

        if is_in_gamut(&test_rgb) {
            low_chroma = mid;
        } else {
            high_chroma = mid;
        }
    }

    let final_scale = if (lab.a * lab.a + lab.b * lab.b).sqrt() > 0.0 {
        low_chroma / (lab.a * lab.a + lab.b * lab.b).sqrt()
    } else {
        0.0
    };

    Lab {
        l: lab.l,
        a: lab.a * final_scale,
        b: lab.b * final_scale,
    }.to_srgb()
}

fn is_in_gamut(rgb: &Rgb) -> bool {
    rgb.r >= 0.0 && rgb.r <= 1.0 &&
    rgb.g >= 0.0 && rgb.g <= 1.0 &&
    rgb.b >= 0.0 && rgb.b <= 1.0
}
```

### Contrast and Accessibility

```rust
/// WCAG 2.1 relative luminance
fn relative_luminance(rgb: &Rgb) -> f32 {
    let linear = rgb_srgb_to_linear(rgb);
    0.2126 * linear.r + 0.7152 * linear.g + 0.0722 * linear.b
}

/// WCAG contrast ratio (1:1 to 21:1)
fn contrast_ratio(c1: &Rgb, c2: &Rgb) -> f32 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// WCAG requirements:
/// - Normal text: 4.5:1 (AA), 7:1 (AAA)
/// - Large text: 3:1 (AA), 4.5:1 (AAA)
fn meets_wcag_aa(foreground: &Rgb, background: &Rgb, large_text: bool) -> bool {
    let ratio = contrast_ratio(foreground, background);
    if large_text { ratio >= 3.0 } else { ratio >= 4.5 }
}
```

---

## Quick Reference

| Conversion | Use Case |
|------------|----------|
| RGB ↔ Linear RGB | Blending, interpolation |
| RGB ↔ HSV/HSL | Color picker, adjustments |
| RGB ↔ Lab | Perceptual operations |
| RGB ↔ Oklab | Modern perceptual work |
| RGB ↔ XYZ | Profile conversions |
| RGB ↔ CMYK | Print output (use ICC!) |

| Color Space | Perceptually Uniform | Gamut |
|-------------|---------------------|-------|
| sRGB | No | ~35% of visible |
| Adobe RGB | No | ~50% of visible |
| Lab | Approximately | Unbounded |
| Oklab | Yes | Unbounded |
| Display P3 | No | ~45% of visible |
