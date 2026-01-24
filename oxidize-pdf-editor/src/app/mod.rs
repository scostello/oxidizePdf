pub mod message;
pub mod state;

use crate::pdf::{DocumentHandle, DocumentMetadata};
use crate::ui;
use iced::{Element, Task, Theme};
use message::Message;
use state::EditorState;
use std::path::PathBuf;

/// Main application state
#[derive(Default)]
pub struct PdfEditor {
    state: EditorState,
}

/// Initialize the application
pub fn new() -> (PdfEditor, Task<Message>) {
    (PdfEditor::default(), Task::none())
}

/// Get the window title
pub fn title(editor: &PdfEditor) -> String {
    if let Some(doc) = &editor.state.current_document {
        format!("oxidizePdf Editor - {}", doc.filename())
    } else {
        "oxidizePdf Editor".to_string()
    }
}

/// Update the application state based on messages
pub fn update(editor: &mut PdfEditor, message: Message) -> Task<Message> {
    match message {
        Message::OpenFile => {
            // Open file dialog asynchronously
            Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("PDF", &["pdf"])
                        .set_title("Open PDF")
                        .pick_file()
                        .await
                },
                |file| {
                    if let Some(file) = file {
                        Message::FileOpened(Ok(file.path().to_path_buf()))
                    } else {
                        Message::FileOpened(Err("No file selected".to_string()))
                    }
                },
            )
        }

        Message::FileOpened(Ok(path)) => {
            // Load document asynchronously
            editor.state.is_loading = true;
            editor.state.error_message = None;

            Task::perform(load_document_async(path), Message::DocumentLoaded)
        }

        Message::FileOpened(Err(error)) => {
            tracing::warn!("File selection cancelled or failed: {}", error);
            Task::none()
        }

        Message::DocumentLoaded(Ok(metadata)) => {
            editor.state.is_loading = false;
            editor.state.total_pages = metadata.page_count;
            editor.state.current_document = Some(metadata);
            editor.state.current_page_index = 0;
            Task::none()
        }

        Message::DocumentLoaded(Err(error)) => {
            editor.state.is_loading = false;
            editor.state.error_message = Some(error);
            editor.state.current_document = None;
            editor.state.total_pages = 0;
            editor.state.current_page_index = 0;
            Task::none()
        }

        Message::NextPage => {
            if editor.state.can_go_next() {
                editor.state.current_page_index += 1;
            }
            Task::none()
        }

        Message::PreviousPage => {
            if editor.state.can_go_previous() {
                editor.state.current_page_index = editor.state.current_page_index.saturating_sub(1);
            }
            Task::none()
        }

        Message::GoToPage(index) => {
            if index < editor.state.total_pages {
                editor.state.current_page_index = index;
            }
            Task::none()
        }
    }
}

/// Render the view
pub fn view(editor: &PdfEditor) -> Element<'_, Message> {
    ui::view(&editor.state)
}

/// Get the theme
pub fn theme(_editor: &PdfEditor) -> Theme {
    Theme::TokyoNightStorm
}

/// Load a document asynchronously (runs in tokio runtime)
async fn load_document_async(path: PathBuf) -> Result<DocumentMetadata, String> {
    tokio::task::spawn_blocking(move || match DocumentHandle::open(path) {
        Ok(doc) => {
            let metadata = doc.extract_metadata();
            tracing::info!(
                "Successfully loaded PDF: {} ({} pages)",
                metadata.filename(),
                metadata.page_count
            );
            Ok(metadata)
        }
        Err(e) => Err(format!("Failed to load PDF: {}", e)),
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
