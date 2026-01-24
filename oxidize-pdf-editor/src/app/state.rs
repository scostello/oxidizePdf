use crate::pdf::DocumentMetadata;

/// Application state
#[derive(Default)]
pub struct EditorState {
    /// Currently loaded document metadata
    pub current_document: Option<DocumentMetadata>,

    /// Current page index (0-based)
    pub current_page_index: usize,

    /// Total number of pages in current document
    pub total_pages: usize,

    /// Error message to display (if any)
    pub error_message: Option<String>,

    /// Loading state
    pub is_loading: bool,
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a document is loaded
    pub fn has_document(&self) -> bool {
        self.current_document.is_some()
    }

    /// Check if can go to next page
    pub fn can_go_next(&self) -> bool {
        self.has_document() && self.current_page_index < self.total_pages.saturating_sub(1)
    }

    /// Check if can go to previous page
    pub fn can_go_previous(&self) -> bool {
        self.has_document() && self.current_page_index > 0
    }

    /// Get current page number (1-based for display)
    pub fn current_page_display(&self) -> usize {
        if self.has_document() {
            self.current_page_index + 1
        } else {
            0
        }
    }
}
