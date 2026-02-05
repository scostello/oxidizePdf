use crate::app::{message::Message, state::EditorState};
use iced::widget::{button, column, container, image, row, scrollable, text, Column};
use iced::{Element, Length};

/// Create the main view for the application
pub fn view(state: &EditorState) -> Element<'_, Message> {
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
    container(text("Loading PDF...").size(24))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Error view
fn error_view(error: &str) -> Element<'_, Message> {
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
fn document_view(state: &EditorState) -> Element<'_, Message> {
    let doc = state.current_document.as_ref().unwrap();

    // Create zoom controls
    let zoom_out_btn = button("-").on_press_maybe(if state.can_zoom_out() {
        Some(Message::ZoomOut)
    } else {
        None
    });

    let zoom_in_btn = button("+").on_press_maybe(if state.can_zoom_in() {
        Some(Message::ZoomIn)
    } else {
        None
    });

    let zoom_display = text(format!("{}%", state.zoom_percentage()));

    let zoom_controls = row![zoom_out_btn, zoom_display, zoom_in_btn]
        .spacing(5)
        .align_y(iced::Alignment::Center);

    // Create navigation controls
    let prev_button = button("◀ Prev").on_press_maybe(if state.can_go_previous() {
        Some(Message::PreviousPage)
    } else {
        None
    });

    let next_button = button("Next ▶").on_press_maybe(if state.can_go_next() {
        Some(Message::NextPage)
    } else {
        None
    });

    let page_info = text(format!(
        "Page {} of {}",
        state.current_page_display(),
        state.total_pages
    ));

    // Toolbar with navigation and zoom
    let toolbar = row![
        prev_button,
        page_info,
        next_button,
        container(row![]).width(Length::Fill), // Spacer
        zoom_controls,
        button("Open").on_press(Message::OpenFile)
    ]
    .spacing(10)
    .padding(10)
    .align_y(iced::Alignment::Center);

    // Page content - either rendered image or loading indicator
    let page_content: Element<'_, Message> = if state.is_rendering {
        container(text("Rendering...").size(18))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    } else if let Some(handle) = &state.rendered_page {
        // Show rendered page in a scrollable container for pan
        scrollable(
            container(image(handle.clone()))
                .padding(20)
                .center_x(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        container(text("No page rendered").size(14))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    };

    // Info bar at bottom
    let info_bar = row![
        text(format!("File: {}", doc.filename())).size(12),
        container(row![]).width(Length::Fill), // Spacer
        text(format!("PDF {}", doc.version)).size(12),
    ]
    .spacing(10)
    .padding(5);

    // Main layout
    container(
        column![toolbar, page_content, info_bar].spacing(0),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
