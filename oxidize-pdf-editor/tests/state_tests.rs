//! Unit tests for EditorState

use oxidize_pdf_editor::app::state::{EditorState, DEFAULT_ZOOM, ZOOM_PRESETS};

#[test]
fn test_default_state() {
    let state = EditorState::default();

    assert!(state.current_document.is_none());
    assert!(state.pdf_path.is_none());
    assert_eq!(state.current_page_index, 0);
    assert_eq!(state.total_pages, 0);
    assert!(state.error_message.is_none());
    assert!(!state.is_loading);
    assert!(state.rendered_page.is_none());
    assert!(!state.is_rendering);
    assert_eq!(state.zoom_level, DEFAULT_ZOOM);
}

#[test]
fn test_has_document() {
    let mut state = EditorState::default();
    assert!(!state.has_document());

    // Simulate loading a document
    state.current_document = Some(create_test_metadata());
    assert!(state.has_document());
}

#[test]
fn test_navigation_no_document() {
    let state = EditorState::default();

    assert!(!state.can_go_next());
    assert!(!state.can_go_previous());
}

#[test]
fn test_navigation_single_page() {
    let mut state = EditorState::default();
    state.current_document = Some(create_test_metadata());
    state.total_pages = 1;
    state.current_page_index = 0;

    assert!(!state.can_go_next());
    assert!(!state.can_go_previous());
}

#[test]
fn test_navigation_multiple_pages() {
    let mut state = EditorState::default();
    state.current_document = Some(create_test_metadata());
    state.total_pages = 5;

    // At first page
    state.current_page_index = 0;
    assert!(state.can_go_next());
    assert!(!state.can_go_previous());

    // At middle page
    state.current_page_index = 2;
    assert!(state.can_go_next());
    assert!(state.can_go_previous());

    // At last page
    state.current_page_index = 4;
    assert!(!state.can_go_next());
    assert!(state.can_go_previous());
}

#[test]
fn test_current_page_display() {
    let mut state = EditorState::default();

    // No document
    assert_eq!(state.current_page_display(), 0);

    // With document
    state.current_document = Some(create_test_metadata());
    state.current_page_index = 0;
    assert_eq!(state.current_page_display(), 1); // 1-based

    state.current_page_index = 4;
    assert_eq!(state.current_page_display(), 5);
}

#[test]
fn test_zoom_percentage() {
    let mut state = EditorState::default();

    state.zoom_level = 1.0;
    assert_eq!(state.zoom_percentage(), 100);

    state.zoom_level = 0.5;
    assert_eq!(state.zoom_percentage(), 50);

    state.zoom_level = 2.0;
    assert_eq!(state.zoom_percentage(), 200);
}

#[test]
fn test_zoom_limits() {
    let mut state = EditorState::default();

    // At default zoom (1.0)
    assert!(state.can_zoom_in());
    assert!(state.can_zoom_out());

    // At minimum zoom
    state.zoom_level = *ZOOM_PRESETS.first().unwrap();
    assert!(state.can_zoom_in());
    assert!(!state.can_zoom_out());

    // At maximum zoom
    state.zoom_level = *ZOOM_PRESETS.last().unwrap();
    assert!(!state.can_zoom_in());
    assert!(state.can_zoom_out());
}

#[test]
fn test_next_zoom_level() {
    let mut state = EditorState::default();

    state.zoom_level = 1.0;
    assert_eq!(state.next_zoom_level(), 1.25);

    state.zoom_level = 0.5;
    assert_eq!(state.next_zoom_level(), 0.75);

    // At max, stays the same
    state.zoom_level = *ZOOM_PRESETS.last().unwrap();
    assert_eq!(state.next_zoom_level(), state.zoom_level);
}

#[test]
fn test_prev_zoom_level() {
    let mut state = EditorState::default();

    state.zoom_level = 1.0;
    assert_eq!(state.prev_zoom_level(), 0.75);

    state.zoom_level = 2.0;
    assert_eq!(state.prev_zoom_level(), 1.5);

    // At min, stays the same
    state.zoom_level = *ZOOM_PRESETS.first().unwrap();
    assert_eq!(state.prev_zoom_level(), state.zoom_level);
}

// Helper to create test metadata
fn create_test_metadata() -> oxidize_pdf_editor::pdf::DocumentMetadata {
    oxidize_pdf_editor::pdf::DocumentMetadata {
        path: std::path::PathBuf::from("/test/file.pdf"),
        page_count: 10,
        title: None,
        author: None,
        version: "1.7".to_string(),
    }
}
