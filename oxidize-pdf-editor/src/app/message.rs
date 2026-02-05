use std::path::PathBuf;

/// Messages for the Elm architecture pattern
#[derive(Debug, Clone)]
pub enum Message {
    /// Open file dialog
    OpenFile,

    /// File was selected (or dialog cancelled)
    FileOpened(Result<PathBuf, String>),

    /// Document was loaded successfully or with error
    DocumentLoaded(Result<crate::pdf::DocumentMetadata, String>),

    /// Navigate to next page
    NextPage,

    /// Navigate to previous page
    PreviousPage,

    /// Go to specific page (0-based index)
    #[allow(dead_code)]
    GoToPage(usize),

    /// Page was rendered (contains image handle or error)
    PageRendered(Result<iced::widget::image::Handle, String>),

    /// Zoom in one step
    ZoomIn,

    /// Zoom out one step
    ZoomOut,

    /// Set zoom to specific level
    ZoomSet(f32),
}
