mod app;
mod pdf;
mod ui;

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting oxidizePdf Editor - Phase 1: Basic Viewer");

    // Run the application using the builder pattern
    iced::application("oxidizePdf Editor", app::update, app::view)
        .theme(app::theme)
        .run_with(app::new)
}
