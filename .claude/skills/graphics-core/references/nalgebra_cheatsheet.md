# nalgebra Cheatsheet

Complete API reference for nalgebra in graphics programming contexts.

## Table of Contents

1. [Type Aliases](#type-aliases)
2. [Vector Operations](#vector-operations)
3. [Matrix Operations](#matrix-operations)
4. [Points vs Vectors](#points-vs-vectors)
5. [Transformations](#transformations)
6. [Rotations](#rotations)
7. [Projections](#projections)
8. [Interpolation](#interpolation)
9. [Geometry Types](#geometry-types)
10. [Conversion Patterns](#conversion-patterns)

---

## Type Aliases

### Common Vectors

```rust
use nalgebra::{Vector2, Vector3, Vector4};

type Vec2 = Vector2<f32>;
type Vec3 = Vector3<f32>;
type Vec4 = Vector4<f32>;

type DVec2 = Vector2<f64>;  // Double precision
type DVec3 = Vector3<f64>;
```

### Common Matrices

```rust
use nalgebra::{Matrix2, Matrix3, Matrix4};
use nalgebra::{Matrix2x3, Matrix3x4};  // Non-square

type Mat2 = Matrix2<f32>;
type Mat3 = Matrix3<f32>;
type Mat4 = Matrix4<f32>;
```

### Points

```rust
use nalgebra::{Point2, Point3};

type Pt2 = Point2<f32>;
type Pt3 = Point3<f32>;
```

---

## Vector Operations

### Construction

```rust
// From components
let v = Vector3::new(1.0, 2.0, 3.0);

// From array
let v = Vector3::from([1.0, 2.0, 3.0]);

// From slice
let v = Vector3::from_row_slice(&[1.0, 2.0, 3.0]);

// Unit vectors
let x = Vector3::x();        // (1, 0, 0)
let y = Vector3::y();        // (0, 1, 0)
let z = Vector3::z();        // (0, 0, 1)

// Zero and ones
let zero = Vector3::zeros();
let ones = Vector3::from_element(1.0);

// Repeat value
let v = Vector3::repeat(5.0);  // (5, 5, 5)
```

### Accessing Components

```rust
let v = Vector3::new(1.0, 2.0, 3.0);

// By index
let x = v[0];
let y = v[1];
let z = v[2];

// By name
let x = v.x;
let y = v.y;
let z = v.z;

// As array/slice
let arr: [f32; 3] = v.into();
let slice: &[f32] = v.as_slice();

// Swizzling (manual)
let xy = Vector2::new(v.x, v.y);
let xz = Vector2::new(v.x, v.z);
```

### Arithmetic

```rust
let a = Vector3::new(1.0, 2.0, 3.0);
let b = Vector3::new(4.0, 5.0, 6.0);

// Addition/subtraction
let sum = a + b;
let diff = a - b;

// Scalar multiplication/division
let scaled = a * 2.0;
let divided = a / 2.0;

// Component-wise multiplication
let product = a.component_mul(&b);

// Component-wise division
let quotient = a.component_div(&b);

// Negation
let neg = -a;
```

### Vector Math

```rust
let v = Vector3::new(3.0, 4.0, 0.0);

// Length (magnitude)
let len = v.magnitude();           // 5.0
let len_sq = v.magnitude_squared(); // 25.0 (faster, no sqrt)

// Normalize
let unit = v.normalize();
let unit = v.try_normalize(1e-6);  // Returns Option, handles zero

// Guaranteed unit vector
let unit: Unit<Vector3<f32>> = Unit::new_normalize(v);

// Dot product
let dot = a.dot(&b);

// Cross product (3D only)
let cross = a.cross(&b);

// Angle between vectors
let angle = a.angle(&b);  // radians

// Projection of a onto b
let proj = a.dot(&b) / b.magnitude_squared() * b;

// Reflection
let reflected = a - 2.0 * a.dot(&normal) * normal;
```

### Min/Max/Clamp

```rust
// Component-wise min/max
let min = a.inf(&b);   // min of each component
let max = a.sup(&b);   // max of each component

// Clamp each component
let clamped = v.map(|x| x.clamp(0.0, 1.0));

// Min/max component value
let min_val = v.min();
let max_val = v.max();

// Index of min/max
let min_idx = v.imin();
let max_idx = v.imax();
```

---

## Matrix Operations

### Construction

```rust
// Identity
let i = Matrix3::identity();
let i = Matrix4::identity();

// From rows
let m = Matrix3::from_rows(&[
    RowVector3::new(1.0, 0.0, 0.0),
    RowVector3::new(0.0, 1.0, 0.0),
    RowVector3::new(0.0, 0.0, 1.0),
]);

// From columns
let m = Matrix3::from_columns(&[
    Vector3::new(1.0, 0.0, 0.0),
    Vector3::new(0.0, 1.0, 0.0),
    Vector3::new(0.0, 0.0, 1.0),
]);

// From flat array (column-major)
let m = Matrix3::from_column_slice(&[
    1.0, 0.0, 0.0,  // column 0
    0.0, 1.0, 0.0,  // column 1
    0.0, 0.0, 1.0,  // column 2
]);

// Diagonal
let m = Matrix3::from_diagonal(&Vector3::new(2.0, 3.0, 4.0));

// Zero
let m = Matrix3::zeros();
```

### Accessing Elements

```rust
let m = Matrix3::identity();

// By index (row, col)
let val = m[(0, 1)];
let val = m.index((0, 1));

// Row/column access
let row = m.row(0);
let col = m.column(1);

// Diagonal
let diag = m.diagonal();
```

### Matrix Math

```rust
// Multiplication
let product = a * b;

// Matrix-vector multiplication
let transformed = m * v;

// Transpose
let t = m.transpose();

// Inverse
let inv = m.try_inverse();  // Returns Option

// Determinant
let det = m.determinant();

// Trace
let tr = m.trace();
```

---

## Points vs Vectors

Points represent positions; vectors represent directions/offsets.

```rust
use nalgebra::{Point2, Point3, Vector2, Vector3};

let point = Point3::new(1.0, 2.0, 3.0);
let vector = Vector3::new(1.0, 0.0, 0.0);

// Point + Vector = Point (translate point)
let new_point = point + vector;

// Point - Point = Vector (direction between points)
let direction = point_b - point_a;

// Point - Vector = Point
let new_point = point - vector;

// Convert point to vector (position vector from origin)
let pos_vec = point.coords;

// Convert vector to point
let pt = Point3::from(vector);

// Origin
let origin = Point3::origin();
```

---

## Transformations

### Translation

```rust
use nalgebra::{Translation2, Translation3};

// Create translation
let t = Translation3::new(10.0, 20.0, 30.0);

// From vector
let t = Translation3::from(Vector3::new(10.0, 20.0, 30.0));

// Apply to point
let new_point = t * point;

// Inverse
let inv = t.inverse();

// Get translation vector
let vec = t.vector;
```

### Rotation (2D)

```rust
use nalgebra::Rotation2;

// From angle (radians)
let r = Rotation2::new(std::f32::consts::FRAC_PI_2);  // 90°

// Apply to vector
let rotated = r * vector;

// Get angle
let angle = r.angle();

// Inverse
let inv = r.inverse();

// Matrix form
let mat: Matrix2<f32> = r.into_inner();
```

### Rotation (3D)

```rust
use nalgebra::{Rotation3, Unit, Vector3};

// From axis-angle
let axis = Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0));
let r = Rotation3::from_axis_angle(&axis, std::f32::consts::FRAC_PI_4);

// From Euler angles (roll, pitch, yaw)
let r = Rotation3::from_euler_angles(roll, pitch, yaw);

// Extract Euler angles
let (roll, pitch, yaw) = r.euler_angles();

// Rotate vector to align with another
let r = Rotation3::rotation_between(&from, &to);

// Look at rotation
let r = Rotation3::look_at_rh(&direction, &up);
```

### Scale

```rust
use nalgebra::Scale2;

// Uniform scale
let s = Scale2::new(2.0, 2.0);

// Non-uniform scale
let s = Scale2::new(2.0, 0.5);

// Apply
let scaled = s * vector;
```

### Isometry (Rotation + Translation)

Rigid body transformation - preserves distances and angles.

```rust
use nalgebra::{Isometry2, Isometry3};

// 2D: rotation then translation
let iso = Isometry2::new(
    Vector2::new(10.0, 20.0),  // translation
    std::f32::consts::FRAC_PI_4,  // rotation angle
);

// 3D: from parts
let iso = Isometry3::from_parts(
    Translation3::new(10.0, 20.0, 30.0),
    UnitQuaternion::from_axis_angle(&axis, angle),
);

// Apply
let transformed = iso * point;

// Inverse
let inv = iso.inverse();

// Compose
let combined = iso_a * iso_b;

// Look-at camera
let view = Isometry3::look_at_rh(&eye, &target, &up);
```

### Similarity (Isometry + Uniform Scale)

```rust
use nalgebra::Similarity3;

let sim = Similarity3::from_parts(
    Translation3::new(10.0, 0.0, 0.0),
    UnitQuaternion::identity(),
    2.0,  // uniform scale
);
```

### Affine (Full 2D/3D Transform)

```rust
use nalgebra::Affine2;

// From matrix
let affine = Affine2::from_matrix_unchecked(matrix);

// Compose transforms
let combined = translate * rotate * scale;
```

---

## Rotations

### Unit Quaternion (3D)

Preferred for 3D rotations - avoids gimbal lock.

```rust
use nalgebra::UnitQuaternion;

// From axis-angle
let q = UnitQuaternion::from_axis_angle(&axis, angle);

// From Euler angles
let q = UnitQuaternion::from_euler_angles(roll, pitch, yaw);

// From rotation matrix
let q = UnitQuaternion::from_rotation_matrix(&rot_matrix);

// Identity (no rotation)
let q = UnitQuaternion::identity();

// Rotate vector
let rotated = q * vector;

// Compose rotations
let combined = q1 * q2;

// Inverse
let inv = q.inverse();

// Interpolate (slerp)
let interp = q1.slerp(&q2, t);

// Convert to matrix
let mat: Rotation3<f32> = q.to_rotation_matrix();

// Extract axis-angle
let (axis, angle) = q.axis_angle().unwrap_or((Vector3::z_axis(), 0.0));
```

### Unit Complex (2D)

Efficient 2D rotation using complex numbers.

```rust
use nalgebra::UnitComplex;

// From angle
let c = UnitComplex::new(angle);

// Rotate
let rotated = c * vector;

// Get angle
let angle = c.angle();
```

---

## Projections

### Perspective

```rust
use nalgebra::Perspective3;

let proj = Perspective3::new(
    aspect_ratio,  // width / height
    fov_y,         // vertical field of view in radians
    near,          // near clipping plane
    far,           // far clipping plane
);

// Get 4x4 matrix
let mat = proj.as_matrix();
let mat = proj.to_homogeneous();

// Project point
let projected = proj.project_point(&point);

// Unproject (screen to world)
let world = proj.unproject_point(&screen_point);
```

### Orthographic

```rust
use nalgebra::Orthographic3;

let proj = Orthographic3::new(
    left, right,
    bottom, top,
    near, far,
);

// Get matrix
let mat = proj.as_matrix();
```

---

## Interpolation

### Linear Interpolation (Lerp)

```rust
// Vectors
let lerped = a.lerp(&b, t);  // t in [0, 1]

// Points
let lerped = point_a.lerp(&point_b, t);

// Manual
let lerped = a * (1.0 - t) + b * t;
```

### Spherical Linear Interpolation (Slerp)

For rotations - maintains constant angular velocity.

```rust
// Quaternions
let interp = q1.slerp(&q2, t);

// Also nlerp (faster, slightly less accurate)
let interp = q1.nlerp(&q2, t);
```

---

## Geometry Types

### Useful for collision/intersection

```rust
use nalgebra::geometry::{Point2, Point3};

// Line segment (not built-in, but pattern)
struct Segment<T> {
    start: Point2<T>,
    end: Point2<T>,
}

// Ray
struct Ray<T> {
    origin: Point3<T>,
    direction: Unit<Vector3<T>>,
}

// Plane (point + normal)
struct Plane {
    point: Point3<f32>,
    normal: Unit<Vector3<f32>>,
}

impl Plane {
    fn signed_distance(&self, p: &Point3<f32>) -> f32 {
        (p - self.point).dot(self.normal.as_ref())
    }
}
```

---

## Conversion Patterns

### To/From Arrays

```rust
// Vector to array
let arr: [f32; 3] = v.into();

// Array to vector
let v = Vector3::from([1.0, 2.0, 3.0]);

// Matrix to flat array (column-major)
let flat: [f32; 9] = m.into();
```

### To/From Homogeneous

```rust
// 3D point to 4D homogeneous
let h = point.to_homogeneous();  // [x, y, z, 1]

// 4D to 3D (perspective divide)
let p = Point3::from_homogeneous(h);

// 3x3 to 4x4
let m4 = m3.to_homogeneous();
```

### Between nalgebra and Other Libraries

```rust
// nalgebra → [f32; N]
let arr: [f32; 3] = v.into();

// [f32; N] → nalgebra
let v = Vector3::from(arr);

// For interop with C/GPU:
let ptr = v.as_ptr();
let slice = v.as_slice();
```

---

## Performance Tips

1. **Use `magnitude_squared()` when possible** - avoids sqrt
2. **Use `Unit<Vector>` for normals** - guarantees normalized, no runtime check
3. **Prefer quaternions over Euler angles** - avoids gimbal lock, better interpolation
4. **Use `try_inverse()` instead of `inverse()`** - handles singular matrices gracefully
5. **Pre-multiply transforms** - `MVP = Proj * View * Model`, apply once
6. **Use `Isometry` for rigid transforms** - more efficient than full matrix
