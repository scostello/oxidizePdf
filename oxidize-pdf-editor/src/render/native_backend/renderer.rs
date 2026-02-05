//! Native PDF Renderer
//!
//! Implements the PageRenderer trait using tiny-skia for rasterization.
//! This is the main entry point for the native rendering backend.

use super::graphics_state::GraphicsStateStack;
use super::path_builder::PdfPathBuilder;
use super::text_renderer::TextRenderer;
use super::{PageRenderer, RenderError};

use oxidize_pdf::parser::content::{ContentOperation, ContentParser};
use oxidize_pdf::parser::{PdfDocument, PdfReader};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tiny_skia::{FillRule, Paint, Pixmap, Stroke, Transform};

/// Native Rust PDF renderer using tiny-skia
///
/// This renderer processes PDF content streams and rasterizes them
/// using the tiny-skia 2D graphics library and skrifa for text.
///
/// # Current Limitations
///
/// - Only basic color spaces (DeviceRGB, DeviceGray, DeviceCMYK)
/// - No transparency group support
/// - No pattern/shading support
/// - Text uses fallback system fonts (embedded font extraction WIP)
pub struct NativeBackend {
    /// Background color for rendered pages
    background: tiny_skia::Color,
    /// Text renderer with font management
    text_renderer: TextRenderer,
}

impl NativeBackend {
    /// Create a new NativeBackend with white background
    pub fn new() -> Self {
        Self {
            background: tiny_skia::Color::WHITE,
            text_renderer: TextRenderer::new(),
        }
    }

    /// Create with a custom background color
    pub fn with_background(r: u8, g: u8, b: u8) -> Self {
        Self {
            background: tiny_skia::Color::from_rgba8(r, g, b, 255),
            text_renderer: TextRenderer::new(),
        }
    }

    /// Render a content stream to a pixmap
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

            // Text operators
            ContentOperation::BeginText => {
                state_stack.current_mut().text.text_matrix = Transform::identity();
                state_stack.current_mut().text.text_line_matrix = Transform::identity();
            }
            ContentOperation::EndText => {}
            ContentOperation::SetFont(name, size) => {
                state_stack.current_mut().text.font_name = Some(name.clone());
                state_stack.current_mut().text.font_size = *size;
            }
            ContentOperation::MoveText(tx, ty) => {
                // Td operator: move to start of next line
                let text = &mut state_stack.current_mut().text;
                let new_matrix = Transform::from_translate(*tx, *ty)
                    .post_concat(text.text_line_matrix);
                text.text_matrix = new_matrix;
                text.text_line_matrix = new_matrix;
            }
            ContentOperation::MoveTextSetLeading(tx, ty) => {
                // TD operator: move and set leading
                state_stack.current_mut().text.leading = -*ty;
                let text = &mut state_stack.current_mut().text;
                let new_matrix = Transform::from_translate(*tx, *ty)
                    .post_concat(text.text_line_matrix);
                text.text_matrix = new_matrix;
                text.text_line_matrix = new_matrix;
            }
            ContentOperation::SetTextMatrix(a, b, c, d, e, f) => {
                // Tm operator: set text matrix directly
                let matrix = Transform::from_row(*a, *b, *c, *d, *e, *f);
                state_stack.current_mut().text.text_matrix = matrix;
                state_stack.current_mut().text.text_line_matrix = matrix;
            }
            ContentOperation::NextLine => {
                // T* operator: move to start of next line
                let leading = state_stack.current().text.leading;
                let text = &mut state_stack.current_mut().text;
                let new_matrix = Transform::from_translate(0.0, -leading)
                    .post_concat(text.text_line_matrix);
                text.text_matrix = new_matrix;
                text.text_line_matrix = new_matrix;
            }
            ContentOperation::SetLeading(leading) => {
                state_stack.current_mut().text.leading = *leading;
            }
            ContentOperation::SetCharSpacing(spacing) => {
                state_stack.current_mut().text.char_spacing = *spacing;
            }
            ContentOperation::SetWordSpacing(spacing) => {
                state_stack.current_mut().text.word_spacing = *spacing;
            }
            ContentOperation::SetHorizontalScaling(scale) => {
                state_stack.current_mut().text.horizontal_scaling = *scale;
            }
            ContentOperation::SetTextRise(rise) => {
                state_stack.current_mut().text.rise = *rise;
            }
            ContentOperation::SetTextRenderMode(mode) => {
                state_stack.current_mut().text.render_mode = *mode;
            }
            ContentOperation::ShowText(text_bytes) => {
                self.render_text(text_bytes, state_stack.current(), pixmap);
            }
            ContentOperation::ShowTextArray(elements) => {
                // TJ operator: show text with positioning adjustments
                use oxidize_pdf::parser::content::TextElement;
                for element in elements {
                    match element {
                        TextElement::Text(text_bytes) => {
                            self.render_text(text_bytes, state_stack.current(), pixmap);
                        }
                        TextElement::Spacing(offset) => {
                            // Negative offset moves right (PDF convention)
                            // Offset is in thousandths of a unit of text space
                            let font_size = state_stack.current().text.font_size;
                            let adjustment = -(*offset) * font_size / 1000.0;
                            let text = &mut state_stack.current_mut().text;
                            text.text_matrix = Transform::from_translate(adjustment, 0.0)
                                .post_concat(text.text_matrix);
                        }
                    }
                }
            }
            ContentOperation::NextLineShowText(text_bytes) => {
                // ' operator: T* then Tj
                let leading = state_stack.current().text.leading;
                {
                    let text = &mut state_stack.current_mut().text;
                    let new_matrix = Transform::from_translate(0.0, -leading)
                        .post_concat(text.text_line_matrix);
                    text.text_matrix = new_matrix;
                    text.text_line_matrix = new_matrix;
                }
                self.render_text(text_bytes, state_stack.current(), pixmap);
            }
            ContentOperation::SetSpacingNextLineShowText(aw, ac, text_bytes) => {
                // " operator: set word/char spacing, T*, then Tj
                state_stack.current_mut().text.word_spacing = *aw;
                state_stack.current_mut().text.char_spacing = *ac;
                let leading = state_stack.current().text.leading;
                {
                    let text = &mut state_stack.current_mut().text;
                    let new_matrix = Transform::from_translate(0.0, -leading)
                        .post_concat(text.text_line_matrix);
                    text.text_matrix = new_matrix;
                    text.text_line_matrix = new_matrix;
                }
                self.render_text(text_bytes, state_stack.current(), pixmap);
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

    /// Render text using the text renderer
    fn render_text(
        &self,
        text_bytes: &[u8],
        state: &super::graphics_state::GraphicsState,
        pixmap: &mut Pixmap,
    ) {
        // Get font info from state
        let font_name = state
            .text
            .font_name
            .as_deref()
            .unwrap_or("Helvetica");
        let font_size = state.text.font_size;

        // Calculate text position from text matrix and CTM
        // Text matrix gives position in user space, CTM transforms to device space
        let text_matrix = state.text.text_matrix;
        let ctm = state.ctm;

        // Combine text matrix with CTM to get device position
        // The text position is (0, 0) in text space, transformed by text matrix
        let tx = text_matrix.tx;
        let ty = text_matrix.ty;

        // Apply CTM to get device coordinates
        let (x, y) = (
            ctm.sx * tx + ctm.kx * ty + ctm.tx,
            ctm.ky * tx + ctm.sy * ty + ctm.ty,
        );

        // Create paint with fill color
        let mut paint = Paint::default();
        paint.set_color(state.fill_color);
        paint.anti_alias = true;

        // Render the text
        // Note: CTM is already applied in the transform calculation above,
        // so we pass identity transform to the text renderer
        self.text_renderer.render_text_with_advances(
            text_bytes,
            font_name,
            font_size,
            x,
            y,
            Transform::identity(),
            &paint,
            pixmap,
        );
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
        // Open PDF with oxidize-pdf using PdfDocument (not PdfReader directly)
        let file = File::open(pdf_path)
            .map_err(|e| RenderError::LoadError(e.to_string()))?;
        let buf_reader = BufReader::new(file);
        let reader = PdfReader::new(buf_reader)
            .map_err(|e| RenderError::LoadError(e.to_string()))?;
        let document = PdfDocument::new(reader);

        // Get page
        let page_idx = page_index as u32;
        let page = document
            .get_page(page_idx)
            .map_err(|_| RenderError::PageNotFound(page_index))?;

        // Get dimensions from page
        let width = page.width() as f32;
        let height = page.height() as f32;

        // Calculate pixel dimensions
        let pixel_width = (width * scale) as u32;
        let pixel_height = (height * scale) as u32;

        // Create pixmap
        let mut pixmap = Pixmap::new(pixel_width, pixel_height)
            .ok_or_else(|| RenderError::RenderFailed("Failed to create pixmap".to_string()))?;

        // Fill with background
        pixmap.fill(self.background);

        // Build transform: scale + flip Y for PDF coordinate system
        // PDF origin is bottom-left, pixmap origin is top-left
        let base_transform = Transform::from_row(
            scale,
            0.0,
            0.0,
            -scale,
            0.0,
            pixel_height as f32,
        );

        // Get and parse content streams
        if let Ok(streams) = page.content_streams_with_document(&document) {
            for stream_data in streams {
                if let Ok(operations) = ContentParser::parse(&stream_data) {
                    self.render_content(&operations, &mut pixmap, base_transform);
                }
            }
        }

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
        let file = File::open(pdf_path)
            .map_err(|e| RenderError::LoadError(e.to_string()))?;
        let buf_reader = BufReader::new(file);
        let reader = PdfReader::new(buf_reader)
            .map_err(|e| RenderError::LoadError(e.to_string()))?;
        let document = PdfDocument::new(reader);

        let page_idx = page_index as u32;
        let page = document
            .get_page(page_idx)
            .map_err(|_| RenderError::PageNotFound(page_index))?;

        Ok((page.width() as f32, page.height() as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxidize_pdf::parser::content::ContentOperation;

    #[test]
    fn test_native_backend_creation() {
        let backend = NativeBackend::new();
        assert_eq!(backend.background, tiny_skia::Color::WHITE);
    }

    #[test]
    fn test_custom_background() {
        let backend = NativeBackend::with_background(200, 200, 200);
        // tiny_skia::Color uses f32 values (0.0-1.0), 200/255 ≈ 0.784
        assert!((backend.background.red() - 200.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn test_render_rectangle() {
        let backend = NativeBackend::new();
        let mut pixmap = Pixmap::new(100, 100).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);

        // Create operations for a black filled rectangle
        let operations = vec![
            ContentOperation::SetNonStrokingGray(0.0), // Black fill
            ContentOperation::Rectangle(10.0, 10.0, 30.0, 30.0),
            ContentOperation::Fill,
        ];

        // Transform: scale 1.0, flip Y (origin at bottom-left)
        let transform = Transform::from_row(1.0, 0.0, 0.0, -1.0, 0.0, 100.0);
        backend.render_content(&operations, &mut pixmap, transform);

        // Check that we rendered something (pixel at center of rectangle should be black)
        // The rectangle is at (10, 10) with size (30, 30), center at ~(25, 25) in PDF coords
        // After Y flip: y = 100 - 25 = 75 in pixmap coords
        let pixel = pixmap.pixel(25, 75).unwrap();
        assert_eq!(pixel.red(), 0, "Rectangle center should be black");
        assert_eq!(pixel.green(), 0);
        assert_eq!(pixel.blue(), 0);
    }

    #[test]
    fn test_render_stroked_path() {
        let backend = NativeBackend::new();
        let mut pixmap = Pixmap::new(100, 100).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);

        // Create operations for a red stroked line
        let operations = vec![
            ContentOperation::SetStrokingRGB(1.0, 0.0, 0.0), // Red stroke
            ContentOperation::SetLineWidth(4.0),
            ContentOperation::MoveTo(10.0, 50.0),
            ContentOperation::LineTo(90.0, 50.0),
            ContentOperation::Stroke,
        ];

        let transform = Transform::from_row(1.0, 0.0, 0.0, -1.0, 0.0, 100.0);
        backend.render_content(&operations, &mut pixmap, transform);

        // Check pixel in the middle of the line (y = 100 - 50 = 50 in pixmap)
        let pixel = pixmap.pixel(50, 50).unwrap();
        assert!(pixel.red() > 200, "Line should be red (r={})", pixel.red());
        assert!(pixel.green() < 50, "Line should be red (g={})", pixel.green());
    }

    #[test]
    fn test_graphics_state_save_restore() {
        let backend = NativeBackend::new();
        let mut pixmap = Pixmap::new(100, 100).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);

        // Draw nested graphics states with different colors
        let operations = vec![
            ContentOperation::SetNonStrokingGray(0.0), // Black
            ContentOperation::SaveGraphicsState,
            ContentOperation::SetNonStrokingRGB(1.0, 0.0, 0.0), // Red
            ContentOperation::Rectangle(40.0, 40.0, 20.0, 20.0),
            ContentOperation::Fill,
            ContentOperation::RestoreGraphicsState,
            // After restore, should be back to black
            ContentOperation::Rectangle(10.0, 10.0, 20.0, 20.0),
            ContentOperation::Fill,
        ];

        let transform = Transform::from_row(1.0, 0.0, 0.0, -1.0, 0.0, 100.0);
        backend.render_content(&operations, &mut pixmap, transform);

        // Check red rectangle (center at 50, 50 -> y = 100 - 50 = 50)
        let red_pixel = pixmap.pixel(50, 50).unwrap();
        assert!(red_pixel.red() > 200, "Should be red");

        // Check black rectangle (center at 20, 20 -> y = 100 - 20 = 80)
        let black_pixel = pixmap.pixel(20, 80).unwrap();
        assert_eq!(black_pixel.red(), 0, "Should be black");
    }

    #[test]
    fn test_transform_matrix() {
        let backend = NativeBackend::new();
        let mut pixmap = Pixmap::new(200, 200).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);

        // Draw a rectangle with a scale transform
        // The transform is applied to subsequent path operations
        let operations = vec![
            ContentOperation::SetNonStrokingGray(0.0),
            ContentOperation::SetTransformMatrix(2.0, 0.0, 0.0, 2.0, 0.0, 0.0), // 2x scale
            ContentOperation::Rectangle(20.0, 20.0, 40.0, 40.0), // With 2x scale: 40,40 to 120,120
            ContentOperation::Fill,
        ];

        // Use identity base transform for this test (no flip)
        let transform = Transform::from_row(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        backend.render_content(&operations, &mut pixmap, transform);

        // After 2x scale, rectangle at (20,20,40,40) becomes (40,40,80,80)
        // Check that there are some non-white pixels in the rendered area
        let mut found_black = false;
        for y in 40..120 {
            for x in 40..120 {
                if let Some(pixel) = pixmap.pixel(x, y) {
                    if pixel.red() == 0 {
                        found_black = true;
                        break;
                    }
                }
            }
            if found_black {
                break;
            }
        }
        assert!(found_black, "Transformed rectangle should contain black pixels");
    }
}
