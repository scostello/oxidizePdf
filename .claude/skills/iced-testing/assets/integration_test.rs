// Integration test template
// Place this file in tests/ directory (not src/)
// Tests complete workflows end-to-end

use my_app::{App, Message, State};
use iced_test::simulator;

#[test]
fn test_complete_workflow() {
    let mut app = App::new();

    // Phase 1: Initialization
    assert_eq!(app.state, State::Initial);

    // Phase 2: Load data
    app.update(Message::LoadData);
    assert_eq!(app.state, State::Loading);

    app.update(Message::DataLoaded(sample_data()));
    assert_eq!(app.state, State::Ready);
    assert!(!app.data.is_empty());

    // Phase 3: User interactions
    let mut ui = simulator(app.view());
    let _ = ui.click("Process");

    for message in ui.into_messages() {
        app.update(message);
    }

    assert_eq!(app.state, State::Processing);

    // Phase 4: Completion
    app.update(Message::ProcessComplete);
    assert_eq!(app.state, State::Complete);
}

#[test]
fn test_error_recovery_workflow() {
    let mut app = App::new();

    // Trigger error
    app.update(Message::LoadError("Network error".into()));
    assert!(matches!(app.state, State::Error(_)));

    // User retries
    let mut ui = simulator(app.view());
    let _ = ui.click("Retry");

    for message in ui.into_messages() {
        app.update(message);
    }

    // Should be back to loading
    assert_eq!(app.state, State::Loading);

    // Success on retry
    app.update(Message::DataLoaded(sample_data()));
    assert_eq!(app.state, State::Ready);
}

#[test]
fn test_multi_file_workflow() {
    let mut app = App::new();

    // Open first file
    app.update(Message::FileSelected("file1.pdf".into()));
    assert_eq!(app.current_file, Some("file1.pdf".into()));

    // Perform operation
    let mut ui = simulator(app.view());
    let _ = ui.click("Rotate");

    for message in ui.into_messages() {
        app.update(message);
    }

    assert!(app.has_unsaved_changes);

    // Save
    let mut ui = simulator(app.view());
    let _ = ui.click("Save");

    for message in ui.into_messages() {
        app.update(message);
    }

    assert!(!app.has_unsaved_changes);

    // Open second file
    app.update(Message::FileSelected("file2.pdf".into()));
    assert_eq!(app.current_file, Some("file2.pdf".into()));
}

// Helper functions
fn sample_data() -> Vec<String> {
    vec!["item1".into(), "item2".into(), "item3".into()]
}
