use std::path::PathBuf;

/// Lightweight metadata extracted from a PDF document
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    /// Path to the PDF file
    pub path: PathBuf,

    /// Number of pages
    pub page_count: usize,

    /// Document title (if any)
    pub title: Option<String>,

    /// Document author (if any)
    pub author: Option<String>,

    /// PDF version
    pub version: String,
}

impl DocumentMetadata {
    pub fn filename(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string()
    }
}
