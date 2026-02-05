//! PDFium-based rendering backend
//!
//! Uses Google's PDFium library via pdfium-render for high-quality PDF rendering.

use super::{PageRenderer, RenderError};
use pdfium_render::prelude::*;
use std::path::Path;

/// PDFium-based PDF renderer
///
/// This backend uses Google's PDFium library (the same renderer used in Chrome)
/// to produce high-quality page renders.
///
/// Note: PDFium is not thread-safe, so this creates a new instance per operation.
/// For better performance with repeated renders, consider caching at a higher level.
pub struct PdfiumBackend {
    library_path: Option<String>,
}

impl PdfiumBackend {
    /// Create a new PdfiumBackend that will search for pdfium in default locations
    pub fn new() -> Self {
        Self { library_path: None }
    }

    /// Create with a specific library path
    pub fn with_library_path(path: impl Into<String>) -> Self {
        Self {
            library_path: Some(path.into()),
        }
    }

    /// Get a Pdfium instance
    fn get_pdfium(&self) -> Result<Pdfium, RenderError> {
        let bindings = if let Some(ref path) = self.library_path {
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(path))
        } else {
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
        }
        .map_err(|e| RenderError::PdfiumNotFound(e.to_string()))?;

        Ok(Pdfium::new(bindings))
    }
}

impl Default for PdfiumBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PageRenderer for PdfiumBackend {
    fn render_page(
        &self,
        pdf_path: &Path,
        page_index: usize,
        scale: f32,
    ) -> Result<image::RgbaImage, RenderError> {
        let pdfium = self.get_pdfium()?;

        // Load the document
        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| RenderError::LoadError(e.to_string()))?;

        // Get the page
        let page = document
            .pages()
            .get(page_index as u16)
            .map_err(|_| RenderError::PageNotFound(page_index))?;

        // Calculate dimensions at the given scale
        let width = page.width();
        let height = page.height();

        // Render the page
        let render_config = PdfRenderConfig::new()
            .set_target_width((width.value * scale) as i32)
            .set_target_height((height.value * scale) as i32)
            .render_form_data(true)
            .render_annotations(true);

        let bitmap = page
            .render_with_config(&render_config)
            .map_err(|e| RenderError::RenderFailed(e.to_string()))?;

        // Convert to image::RgbaImage
        let image = bitmap
            .as_image()
            .as_rgba8()
            .ok_or_else(|| RenderError::RenderFailed("Failed to convert to RGBA".to_string()))?
            .clone();

        Ok(image)
    }

    fn page_dimensions(
        &self,
        pdf_path: &Path,
        page_index: usize,
    ) -> Result<(f32, f32), RenderError> {
        let pdfium = self.get_pdfium()?;

        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| RenderError::LoadError(e.to_string()))?;

        let page = document
            .pages()
            .get(page_index as u16)
            .map_err(|_| RenderError::PageNotFound(page_index))?;

        Ok((page.width().value, page.height().value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdfium_backend_creation() {
        // This test will pass even without the library - it only fails on actual use
        let backend = PdfiumBackend::new();
        assert!(backend.library_path.is_none());
    }
}
