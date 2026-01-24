use crate::app::{message::Message, state::EditorState};
use iced::widget::{button, column, container, row, text, Column};
use iced::{Element, Length};

/// Create the main view for the application
pub fn view(state: &EditorState) -> Element<Message> {
    if state.is_loading {
        loading_view()
    } else if let Some(error) = &state.error_message {
        error_view(error)
    } else if state.has_document() {
        document_view(state)
    } else {
        welcome_view()
    }
}

/// Welcome screen shown when no document is loaded
fn welcome_view() -> Element<'static, Message> {
    container(
        column![
            text("oxidizePdf Editor").size(32),
            text("Phase 1: Basic Viewer").size(16),
            button("Open PDF").on_press(Message::OpenFile)
        ]
        .spacing(20)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Loading screen
fn loading_view() -> Element<'static, Message> {
    container(
        text("Loading PDF...").size(24)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Error view
fn error_view(error: &str) -> Element<Message> {
    container(
        column![
            text("Error").size(24),
            text(error).size(14),
            button("Try Again").on_press(Message::OpenFile)
        ]
        .spacing(20)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Document view with metadata and navigation
fn document_view(state: &EditorState) -> Element<Message> {
    let doc = state.current_document.as_ref().unwrap();

    // Create metadata section
    let mut metadata_column = Column::new()
        .spacing(10)
        .padding(20);

    metadata_column = metadata_column.push(text(format!("File: {}", doc.filename())).size(18));

    if let Some(title) = &doc.title {
        metadata_column = metadata_column.push(text(format!("Title: {}", title)));
    }

    if let Some(author) = &doc.author {
        metadata_column = metadata_column.push(text(format!("Author: {}", author)));
    }

    metadata_column = metadata_column.push(text(format!("Pages: {}", state.total_pages)));
    metadata_column = metadata_column.push(text(format!("Version: {}", doc.version)));

    // Create navigation controls
    let prev_button = button("◀ Previous")
        .on_press_maybe(if state.can_go_previous() {
            Some(Message::PreviousPage)
        } else {
            None
        });

    let next_button = button("Next ▶")
        .on_press_maybe(if state.can_go_next() {
            Some(Message::NextPage)
        } else {
            None
        });

    let page_info = text(format!(
        "Page {} of {}",
        state.current_page_display(),
        state.total_pages
    ));

    let navigation = row![
        prev_button,
        container(page_info)
            .width(Length::Fill)
            .center_x(Length::Fill),
        next_button
    ]
    .spacing(10)
    .padding(10);

    // Main layout
    container(
        column![
            // Header
            row![
                text("oxidizePdf Editor").size(24),
                container(button("Open Another PDF").on_press(Message::OpenFile))
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
            ]
            .padding(10)
            .spacing(10),

            // Metadata section
            metadata_column,

            // Page content placeholder
            container(
                text(format!("Page {} content would be displayed here", state.current_page_display()))
                    .size(14)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),

            // Navigation controls
            navigation
        ]
        .spacing(10)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
