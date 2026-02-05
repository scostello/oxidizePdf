use crate::pdf::DocumentMetadata;
use std::path::PathBuf;

/// Zoom presets available in the UI
pub const ZOOM_PRESETS: &[f32] = &[0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0];

/// Default zoom level (100%)
pub const DEFAULT_ZOOM: f32 = 1.0;

/// Application state
pub struct EditorState {
    /// Currently loaded document metadata
    pub current_document: Option<DocumentMetadata>,

    /// Path to the current PDF file (needed for rendering)
    pub pdf_path: Option<PathBuf>,

    /// Current page index (0-based)
    pub current_page_index: usize,

    /// Total number of pages in current document
    pub total_pages: usize,

    /// Error message to display (if any)
    pub error_message: Option<String>,

    /// Loading state
    pub is_loading: bool,

    /// Rendered page image
    pub rendered_page: Option<iced::widget::image::Handle>,

    /// Whether page is currently being rendered
    pub is_rendering: bool,

    /// Current zoom level (1.0 = 100%)
    pub zoom_level: f32,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            current_document: None,
            pdf_path: None,
            current_page_index: 0,
            total_pages: 0,
            error_message: None,
            is_loading: false,
            rendered_page: None,
            is_rendering: false,
            zoom_level: DEFAULT_ZOOM,
        }
    }
}

impl EditorState {
    #[allow(dead_code)]
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

    /// Get zoom level as percentage for display
    pub fn zoom_percentage(&self) -> u32 {
        (self.zoom_level * 100.0) as u32
    }

    /// Check if can zoom in further
    pub fn can_zoom_in(&self) -> bool {
        self.zoom_level < *ZOOM_PRESETS.last().unwrap_or(&3.0)
    }

    /// Check if can zoom out further
    pub fn can_zoom_out(&self) -> bool {
        self.zoom_level > *ZOOM_PRESETS.first().unwrap_or(&0.5)
    }

    /// Get the next zoom level up
    pub fn next_zoom_level(&self) -> f32 {
        ZOOM_PRESETS
            .iter()
            .find(|&&z| z > self.zoom_level)
            .copied()
            .unwrap_or(self.zoom_level)
    }

    /// Get the next zoom level down
    pub fn prev_zoom_level(&self) -> f32 {
        ZOOM_PRESETS
            .iter()
            .rev()
            .find(|&&z| z < self.zoom_level)
            .copied()
            .unwrap_or(self.zoom_level)
    }
}
