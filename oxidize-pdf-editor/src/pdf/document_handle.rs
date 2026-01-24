use super::DocumentMetadata;
use oxidize_pdf::parser::{PdfReader, PdfDocument};
use std::fs::File;
use std::path::PathBuf;
use anyhow::{Context, Result};

/// Wrapper around oxidizePdf's PdfDocument with additional metadata
pub struct DocumentHandle {
    /// Path to the PDF file
    pub path: PathBuf,

    /// The parsed PDF document
    pub document: PdfDocument<File>,

    /// Cached page count
    page_count: usize,
}

impl DocumentHandle {
    /// Open a PDF file and create a DocumentHandle
    pub fn open(path: PathBuf) -> Result<Self> {
        // Open the file
        let file = File::open(&path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;

        // Create PDF reader
        let reader = PdfReader::new(file)
            .with_context(|| format!("Failed to parse PDF: {}", path.display()))?;

        // Create document
        let document = PdfDocument::new(reader);

        // Get page count
        let page_count = document.page_count()
            .with_context(|| "Failed to get page count")? as usize;

        Ok(Self {
            path,
            document,
            page_count,
        })
    }

    /// Get the number of pages
    pub fn page_count(&self) -> usize {
        self.page_count
    }

    /// Get the filename
    pub fn filename(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string()
    }

    /// Get metadata title
    pub fn title(&self) -> Option<String> {
        self.document.metadata()
            .ok()
            .and_then(|m| m.title)
    }

    /// Get metadata author
    pub fn author(&self) -> Option<String> {
        self.document.metadata()
            .ok()
            .and_then(|m| m.author)
    }

    /// Get PDF version
    pub fn version(&self) -> String {
        self.document.metadata()
            .ok()
            .map(|m| m.version)
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Extract lightweight metadata
    pub fn extract_metadata(&self) -> DocumentMetadata {
        DocumentMetadata {
            path: self.path.clone(),
            page_count: self.page_count,
            title: self.title(),
            author: self.author(),
            version: self.version(),
        }
    }
}
