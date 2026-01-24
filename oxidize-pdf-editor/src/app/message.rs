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
    GoToPage(usize),
}
