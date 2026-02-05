//! Native PDF Renderer
//!
//! Implements the PageRenderer trait using tiny-skia for rasterization.
//! This is the main entry point for the native rendering backend.

use super::graphics_state::GraphicsStateStack;
use super::path_builder::PdfPathBuilder;
use super::{PageRenderer, RenderError};

use oxidize_pdf::parser::content::{ContentOperation, ContentParser};
use oxidize_pdf::PdfReader;
use std::path::Path;
use tiny_skia::{FillRule, Paint, Pixmap, Stroke, Transform};

/// Native Rust PDF renderer using tiny-skia
///
/// This renderer processes PDF content streams and rasterizes them
/// using the tiny-skia 2D graphics library.
///
/// # Current Limitations
///
/// - Text rendering not yet implemented (requires skrifa integration)
/// - Only basic color spaces (DeviceRGB, DeviceGray, DeviceCMYK)
/// - No transparency group support
/// - No pattern/shading support
/// - Content stream rendering is WIP
pub struct NativeBackend {
    /// Background color for rendered pages
    background: tiny_skia::Color,
}

impl NativeBackend {
    /// Create a new NativeBackend with white background
    pub fn new() -> Self {
        Self {
            background: tiny_skia::Color::WHITE,
        }
    }

    /// Create with a custom background color
    pub fn with_background(r: u8, g: u8, b: u8) -> Self {
        Self {
            background: tiny_skia::Color::from_rgba8(r, g, b, 255),
        }
    }

    /// Render a content stream to a pixmap
    #[allow(dead_code)]
    fn render_content(
        &self,
        operations: &[ContentOperation],
        pixmap: &mut Pixmap,
        base_transform: Transform,
    ) {
        let mut state_stack = GraphicsStateStack::new();
        let mut path_builder = PdfPathBuilder::new();

        // Apply base transform (scale + flip Y for PDF coordinate system)
        state_stack.current_mut().ctm = base_transform;

        for op in operations {
            self.execute_operation(op, &mut state_stack, &mut path_builder, pixmap);
        }
    }

    /// Execute a single content stream operation
    fn execute_operation(
        &self,
        op: &ContentOperation,
        state_stack: &mut GraphicsStateStack,
        path_builder: &mut PdfPathBuilder,
        pixmap: &mut Pixmap,
    ) {
        match op {
            // Graphics state operators
            ContentOperation::SaveGraphicsState => {
                state_stack.save();
            }
            ContentOperation::RestoreGraphicsState => {
                state_stack.restore();
            }
            ContentOperation::SetTransformMatrix(a, b, c, d, e, f) => {
                let transform = Transform::from_row(*a, *b, *c, *d, *e, *f);
                state_stack.current_mut().concat_ctm(transform);
            }
            ContentOperation::SetLineWidth(w) => {
                state_stack.current_mut().line_width = *w;
            }
            ContentOperation::SetLineCap(cap) => {
                state_stack.current_mut().set_line_cap(*cap);
            }
            ContentOperation::SetLineJoin(join) => {
                state_stack.current_mut().set_line_join(*join);
            }
            ContentOperation::SetMiterLimit(limit) => {
                state_stack.current_mut().miter_limit = *limit;
            }

            // Color operators
            ContentOperation::SetNonStrokingGray(g) => {
                state_stack.current_mut().set_fill_gray(*g);
            }
            ContentOperation::SetStrokingGray(g) => {
                state_stack.current_mut().set_stroke_gray(*g);
            }
            ContentOperation::SetNonStrokingRGB(r, g, b) => {
                state_stack.current_mut().set_fill_rgb(*r, *g, *b);
            }
            ContentOperation::SetStrokingRGB(r, g, b) => {
                state_stack.current_mut().set_stroke_rgb(*r, *g, *b);
            }
            ContentOperation::SetNonStrokingCMYK(c, m, y, k) => {
                state_stack.current_mut().set_fill_cmyk(*c, *m, *y, *k);
            }
            ContentOperation::SetStrokingCMYK(c, m, y, k) => {
                state_stack.current_mut().set_stroke_cmyk(*c, *m, *y, *k);
            }

            // Path construction operators
            ContentOperation::MoveTo(x, y) => {
                path_builder.move_to(*x, *y);
            }
            ContentOperation::LineTo(x, y) => {
                path_builder.line_to(*x, *y);
            }
            ContentOperation::CurveTo(x1, y1, x2, y2, x3, y3) => {
                path_builder.curve_to(*x1, *y1, *x2, *y2, *x3, *y3);
            }
            ContentOperation::CurveToV(x2, y2, x3, y3) => {
                path_builder.curve_to_v(*x2, *y2, *x3, *y3);
            }
            ContentOperation::CurveToY(x1, y1, x3, y3) => {
                path_builder.curve_to_y(*x1, *y1, *x3, *y3);
            }
            ContentOperation::ClosePath => {
                path_builder.close();
            }
            ContentOperation::Rectangle(x, y, w, h) => {
                path_builder.rectangle(*x, *y, *w, *h);
            }

            // Path painting operators
            ContentOperation::Stroke => {
                self.stroke_path(path_builder, state_stack.current(), pixmap);
            }
            ContentOperation::CloseStroke => {
                path_builder.close();
                self.stroke_path(path_builder, state_stack.current(), pixmap);
            }
            ContentOperation::Fill => {
                self.fill_path(path_builder, state_stack.current(), pixmap, FillRule::Winding);
            }
            ContentOperation::FillEvenOdd => {
                self.fill_path(path_builder, state_stack.current(), pixmap, FillRule::EvenOdd);
            }
            ContentOperation::FillStroke => {
                self.fill_path(path_builder, state_stack.current(), pixmap, FillRule::Winding);
                self.stroke_path(path_builder, state_stack.current(), pixmap);
            }
            ContentOperation::FillStrokeEvenOdd => {
                self.fill_path(path_builder, state_stack.current(), pixmap, FillRule::EvenOdd);
                self.stroke_path(path_builder, state_stack.current(), pixmap);
            }
            ContentOperation::CloseFillStroke => {
                path_builder.close();
                self.fill_path(path_builder, state_stack.current(), pixmap, FillRule::Winding);
                self.stroke_path(path_builder, state_stack.current(), pixmap);
            }
            ContentOperation::CloseFillStrokeEvenOdd => {
                path_builder.close();
                self.fill_path(path_builder, state_stack.current(), pixmap, FillRule::EvenOdd);
                self.stroke_path(path_builder, state_stack.current(), pixmap);
            }
            ContentOperation::EndPath => {
                path_builder.clear();
            }

            // Text operators - TODO: implement with skrifa
            ContentOperation::BeginText => {
                state_stack.current_mut().text.text_matrix = Transform::identity();
                state_stack.current_mut().text.text_line_matrix = Transform::identity();
            }
            ContentOperation::EndText => {}
            ContentOperation::SetFont(name, size) => {
                state_stack.current_mut().text.font_name = Some(name.clone());
                state_stack.current_mut().text.font_size = *size;
            }
            ContentOperation::ShowText(_) | ContentOperation::ShowTextArray(_) => {
                // TODO: Implement text rendering with skrifa
            }

            _ => {}
        }
    }

    /// Fill the current path
    fn fill_path(
        &self,
        path_builder: &mut PdfPathBuilder,
        state: &super::graphics_state::GraphicsState,
        pixmap: &mut Pixmap,
        fill_rule: FillRule,
    ) {
        if let Some(path) = path_builder.build_transformed(state.ctm) {
            let mut paint = Paint::default();
            paint.set_color(state.fill_color);
            paint.anti_alias = true;
            pixmap.fill_path(&path, &paint, fill_rule, Transform::identity(), None);
        }
        path_builder.clear();
    }

    /// Stroke the current path
    fn stroke_path(
        &self,
        path_builder: &mut PdfPathBuilder,
        state: &super::graphics_state::GraphicsState,
        pixmap: &mut Pixmap,
    ) {
        if let Some(path) = path_builder.build_transformed(state.ctm) {
            let mut paint = Paint::default();
            paint.set_color(state.stroke_color);
            paint.anti_alias = true;

            let mut stroke = Stroke::default();
            stroke.width = state.line_width;
            stroke.line_cap = state.line_cap;
            stroke.line_join = state.line_join;
            stroke.miter_limit = state.miter_limit;

            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
        path_builder.clear();
    }
}

impl Default for NativeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PageRenderer for NativeBackend {
    fn render_page(
        &self,
        pdf_path: &Path,
        page_index: usize,
        scale: f32,
    ) -> Result<image::RgbaImage, RenderError> {
        // Open PDF with oxidize-pdf
        let mut reader = PdfReader::open(pdf_path)
            .map_err(|e| RenderError::LoadError(e.to_string()))?;

        // Get page dimensions
        let page_idx = page_index as u32;
        let page = reader
            .get_page(page_idx)
            .map_err(|_| RenderError::PageNotFound(page_index))?;

        let media_box = page.media_box;
        let width = (media_box[2] - media_box[0]).abs() as f32;
        let height = (media_box[3] - media_box[1]).abs() as f32;

        // Calculate pixel dimensions
        let pixel_width = (width * scale) as u32;
        let pixel_height = (height * scale) as u32;

        // Create pixmap
        let mut pixmap = Pixmap::new(pixel_width, pixel_height)
            .ok_or_else(|| RenderError::RenderFailed("Failed to create pixmap".to_string()))?;

        // Fill with background
        pixmap.fill(self.background);

        // TODO: Content stream rendering requires resolving borrow checker issues
        // with PdfReader API. For now, render blank page with correct dimensions.
        // Next step: Use document.get_page_content_streams() pattern

        // Convert pixmap to image::RgbaImage
        let data = pixmap.data().to_vec();
        image::RgbaImage::from_raw(pixel_width, pixel_height, data)
            .ok_or_else(|| RenderError::RenderFailed("Failed to create image".to_string()))
    }

    fn page_dimensions(
        &self,
        pdf_path: &Path,
        page_index: usize,
    ) -> Result<(f32, f32), RenderError> {
        let mut reader = PdfReader::open(pdf_path)
            .map_err(|e| RenderError::LoadError(e.to_string()))?;

        let page_idx = page_index as u32;
        let page = reader
            .get_page(page_idx)
            .map_err(|_| RenderError::PageNotFound(page_index))?;

        let media_box = page.media_box;
        let width = (media_box[2] - media_box[0]).abs() as f32;
        let height = (media_box[3] - media_box[1]).abs() as f32;

        Ok((width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_backend_creation() {
        let backend = NativeBackend::new();
        assert_eq!(backend.background, tiny_skia::Color::WHITE);
    }

    #[test]
    fn test_custom_background() {
        let backend = NativeBackend::with_background(200, 200, 200);
        assert_eq!(backend.background.red(), 200);
    }
}
