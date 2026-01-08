use crate::renderer::{Document, PdfRenderer};
use iced::widget::image::Handle;
use std::collections::HashMap;
use std::path::PathBuf;

/// Manages a loaded PDF document with rendered page cache
#[derive(Debug)]
pub struct PdfDocument {
    path: PathBuf,
    document: Document,
    page_cache: HashMap<(usize, u32), Handle>, // (page_index, zoom_percent) -> rendered image
}

// Manual Debug impl for Document since it contains pdfium types
impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Document")
            .field("page_count", &self.page_count())
            .finish()
    }
}

impl PdfDocument {
    /// Load a PDF document from a file path
    pub async fn load(path: PathBuf) -> Result<Self, String> {
        // Create a renderer for this document
        let renderer = PdfRenderer::new()
            .map_err(|e| format!("Failed to create renderer: {}", e))?;

        let document = renderer
            .load_document(&path)
            .map_err(|e| format!("Failed to load document: {}", e))?;

        Ok(Self {
            path,
            document,
            page_cache: HashMap::new(),
        })
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string()
    }

    pub fn page_count(&self) -> usize {
        self.document.page_count()
    }

    pub fn get_rendered_page(&mut self, page_index: usize, zoom: f32) -> Option<Handle> {
        let zoom_percent = (zoom * 100.0) as u32;
        let cache_key = (page_index, zoom_percent);

        // Check cache first
        if let Some(handle) = self.page_cache.get(&cache_key) {
            return Some(handle.clone());
        }

        // Render the page
        match self.document.render_page(page_index, zoom) {
            Ok(img) => {
                let width = img.width();
                let height = img.height();
                let rgba = img.into_raw();

                let handle = Handle::from_rgba(width, height, rgba);

                // Cache the rendered page
                self.page_cache.insert(cache_key, handle.clone());

                // Limit cache size to avoid memory issues
                if self.page_cache.len() > 10 {
                    // Remove oldest entries (simple strategy - could be improved with LRU)
                    let keys_to_remove: Vec<_> = self.page_cache.keys()
                        .take(self.page_cache.len() - 10)
                        .cloned()
                        .collect();
                    for key in keys_to_remove {
                        self.page_cache.remove(&key);
                    }
                }

                Some(handle)
            }
            Err(e) => {
                tracing::error!("Failed to render page {}: {}", page_index, e);
                None
            }
        }
    }

    pub fn clear_cache(&mut self) {
        self.page_cache.clear();
    }
}
