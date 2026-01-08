use anyhow::{Context, Result};
use pdfium_render::prelude::*;
use std::path::Path;

/// PDF renderer using pdfium-render
pub struct PdfRenderer {
    pdfium: Pdfium,
}

impl PdfRenderer {
    pub fn new() -> Result<Self> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .context("Failed to bind to PDFium library. Please install PDFium or download the library from https://github.com/bblanchon/pdfium-binaries")?,
        );
        Ok(Self { pdfium })
    }

    pub fn load_document(&self, path: &Path) -> Result<Document> {
        let document = self
            .pdfium
            .load_pdf_from_file(path, None)
            .context("Failed to load PDF document")?;
        Ok(Document {
            inner: document,
        })
    }
}

pub struct Document {
    inner: PdfDocument<'static>,
}

impl Document {
    pub fn page_count(&self) -> usize {
        self.inner.pages().len() as usize
    }

    pub fn render_page(&self, page_index: usize, zoom: f32) -> Result<image::RgbaImage> {
        let page = self
            .inner
            .pages()
            .get(page_index as u16)
            .context("Page index out of bounds")?;

        // Get page dimensions
        let width = page.width();
        let height = page.height();

        // Calculate render size based on zoom
        let render_width = (width.value * zoom) as u32;
        let render_height = (height.value * zoom) as u32;

        // Render the page
        let render_config = PdfRenderConfig::new()
            .set_target_width(render_width as i32)
            .set_maximum_height(render_height as i32)
            .rotate_if_landscape(PdfPageRenderRotation::None, false);

        let bitmap = page
            .render_with_config(&render_config)
            .context("Failed to render page")?;

        // Convert bitmap to image - use as_raw_bytes() instead of deprecated as_bytes()
        let buffer = bitmap.as_raw_bytes();
        let img = image::RgbaImage::from_raw(bitmap.width() as u32, bitmap.height() as u32, buffer.to_vec())
            .context("Failed to create image from bitmap")?;

        Ok(img)
    }

    pub fn get_page_size(&self, page_index: usize) -> Result<(f32, f32)> {
        let page = self
            .inner
            .pages()
            .get(page_index as u16)
            .context("Page index out of bounds")?;
        Ok((page.width().value, page.height().value))
    }
}
