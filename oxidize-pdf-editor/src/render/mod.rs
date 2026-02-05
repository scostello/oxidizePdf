//! PDF page rendering module
//!
//! This module provides a trait-based abstraction for rendering PDF pages to bitmaps.
//!
//! # Backends
//!
//! - `pdfium-render` (default): High-quality rendering via Google's PDFium library
//! - `native-render`: Pure Rust rendering via tiny-skia + skrifa (experimental)
//!
//! # Feature Flags
//!
//! - `pdfium-render`: Enable PDFium backend (default)
//! - `native-render`: Enable native Rust backend

#[cfg(feature = "pdfium-render")]
mod pdfium_backend;

#[cfg(feature = "native-render")]
mod native_backend;

#[cfg(feature = "pdfium-render")]
pub use pdfium_backend::PdfiumBackend;

#[cfg(feature = "native-render")]
pub use native_backend::NativeBackend;

use std::path::Path;
use thiserror::Error;

/// Errors that can occur during page rendering
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to load PDF: {0}")]
    LoadError(String),

    #[error("Page {0} not found in document")]
    PageNotFound(usize),

    #[error("Rendering failed: {0}")]
    RenderFailed(String),

    #[error("Pdfium library not found: {0}")]
    PdfiumNotFound(String),
}

/// Trait for PDF page rendering backends
///
/// Implementations convert PDF pages to RGBA bitmaps that can be displayed in the UI.
pub trait PageRenderer {
    /// Render a specific page of a PDF document
    ///
    /// # Arguments
    /// * `pdf_path` - Path to the PDF file
    /// * `page_index` - Zero-based page index
    /// * `scale` - Scale factor (1.0 = 100%, 2.0 = 200%, etc.)
    ///
    /// # Returns
    /// RGBA image data on success, or a RenderError on failure
    fn render_page(
        &self,
        pdf_path: &Path,
        page_index: usize,
        scale: f32,
    ) -> Result<image::RgbaImage, RenderError>;

    /// Get the dimensions of a page without rendering
    ///
    /// # Returns
    /// (width, height) in points (1/72 inch)
    fn page_dimensions(
        &self,
        pdf_path: &Path,
        page_index: usize,
    ) -> Result<(f32, f32), RenderError>;
}
