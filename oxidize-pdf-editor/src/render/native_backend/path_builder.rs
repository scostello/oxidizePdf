//! PDF Path Builder
//!
//! Converts PDF path construction operators to tiny-skia paths.
//!
//! PDF path operators:
//! - m (moveto)
//! - l (lineto)
//! - c (curveto - cubic bezier)
//! - v (curveto - cubic bezier, initial point replicated)
//! - y (curveto - cubic bezier, final point replicated)
//! - h (closepath)
//! - re (rectangle)

use tiny_skia::{Path, PathBuilder as TinySkiaPathBuilder, Transform};

/// Builds a tiny-skia Path from PDF path operators
#[derive(Debug)]
pub struct PdfPathBuilder {
    builder: TinySkiaPathBuilder,
    /// Current point (for relative operations)
    current_x: f32,
    current_y: f32,
    /// Start point of current subpath (for closepath)
    start_x: f32,
    start_y: f32,
}

impl Default for PdfPathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfPathBuilder {
    /// Create a new path builder
    pub fn new() -> Self {
        Self {
            builder: TinySkiaPathBuilder::new(),
            current_x: 0.0,
            current_y: 0.0,
            start_x: 0.0,
            start_y: 0.0,
        }
    }

    /// Move to a point (m operator)
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.builder.move_to(x, y);
        self.current_x = x;
        self.current_y = y;
        self.start_x = x;
        self.start_y = y;
    }

    /// Line to a point (l operator)
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(x, y);
        self.current_x = x;
        self.current_y = y;
    }

    /// Cubic bezier curve (c operator)
    /// From current point to (x3, y3), with control points (x1, y1) and (x2, y2)
    pub fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.builder.cubic_to(x1, y1, x2, y2, x3, y3);
        self.current_x = x3;
        self.current_y = y3;
    }

    /// Cubic bezier with first control point at current point (v operator)
    pub fn curve_to_v(&mut self, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.curve_to(self.current_x, self.current_y, x2, y2, x3, y3);
    }

    /// Cubic bezier with last control point at end point (y operator)
    pub fn curve_to_y(&mut self, x1: f32, y1: f32, x3: f32, y3: f32) {
        self.curve_to(x1, y1, x3, y3, x3, y3);
    }

    /// Close the current subpath (h operator)
    pub fn close(&mut self) {
        self.builder.close();
        self.current_x = self.start_x;
        self.current_y = self.start_y;
    }

    /// Add a rectangle (re operator)
    pub fn rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.move_to(x, y);
        self.line_to(x + width, y);
        self.line_to(x + width, y + height);
        self.line_to(x, y + height);
        self.close();
    }

    /// Finish building and return the path
    pub fn finish(self) -> Option<Path> {
        self.builder.finish()
    }

    /// Finish building and return the path, transformed
    pub fn finish_with_transform(self, transform: Transform) -> Option<Path> {
        self.builder.finish().and_then(|p| p.transform(transform))
    }

    /// Build the path with a transform, without consuming self
    /// Returns the path and clears the builder
    pub fn build_transformed(&mut self, transform: Transform) -> Option<Path> {
        let mut new_builder = TinySkiaPathBuilder::new();
        std::mem::swap(&mut self.builder, &mut new_builder);

        let result = new_builder.finish().and_then(|p| p.transform(transform));

        // Reset state
        self.current_x = 0.0;
        self.current_y = 0.0;
        self.start_x = 0.0;
        self.start_y = 0.0;

        result
    }

    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        // TinySkiaPathBuilder doesn't have is_empty, so we track this ourselves
        // For now, assume not empty if we've done any operations
        self.current_x == 0.0 && self.current_y == 0.0 && self.start_x == 0.0 && self.start_y == 0.0
    }

    /// Clear the path builder for reuse
    pub fn clear(&mut self) {
        self.builder = TinySkiaPathBuilder::new();
        self.current_x = 0.0;
        self.current_y = 0.0;
        self.start_x = 0.0;
        self.start_y = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_and_line() {
        let mut builder = PdfPathBuilder::new();
        builder.move_to(10.0, 20.0);
        builder.line_to(100.0, 200.0);

        let path = builder.finish();
        assert!(path.is_some());
    }

    #[test]
    fn test_rectangle() {
        let mut builder = PdfPathBuilder::new();
        builder.rectangle(0.0, 0.0, 100.0, 50.0);

        let path = builder.finish();
        assert!(path.is_some());

        let path = path.unwrap();
        let bounds = path.bounds();
        assert_eq!(bounds.width(), 100.0);
        assert_eq!(bounds.height(), 50.0);
    }

    #[test]
    fn test_curve() {
        let mut builder = PdfPathBuilder::new();
        builder.move_to(0.0, 0.0);
        builder.curve_to(10.0, 20.0, 30.0, 40.0, 50.0, 50.0);

        let path = builder.finish();
        assert!(path.is_some());
    }

    #[test]
    fn test_closed_path() {
        let mut builder = PdfPathBuilder::new();
        builder.move_to(0.0, 0.0);
        builder.line_to(100.0, 0.0);
        builder.line_to(100.0, 100.0);
        builder.close();

        let path = builder.finish();
        assert!(path.is_some());
    }
}
