use oxidize_pdf_editor::{app, ui};

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting oxidizePdf Editor - Phase 1: Basic Viewer");

    // Run the application using the builder pattern (iced 0.14.0 API)
    iced::application(app::new, app::update, app::view)
        .title(app::title)
        .theme(app::theme)
        .run()
}
