# Iced Testing Guide

This guide covers testing strategies for iced applications using both unit tests and headless UI tests.

## Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Setting Up iced_test](#setting-up-iced_test)
3. [Testing Update Logic](#testing-update-logic)
4. [Headless UI Testing](#headless-ui-testing)
5. [Integration Testing](#integration-testing)
6. [CI/CD Configuration](#cicd-configuration)
7. [Common Patterns](#common-patterns)

## Testing Philosophy

Iced follows the Elm Architecture, which cleanly separates:
- **State** - Application data (Model)
- **Messages** - User interactions and events
- **Update Logic** - Pure functions that transform state
- **View Logic** - UI rendering based on state

This separation enables two testing approaches:
1. **Unit tests** - Test update logic directly (no UI library needed)
2. **Headless tests** - Test UI interactions using `iced_test`

## Setting Up iced_test

Add `iced_test` to your `Cargo.toml`:

```toml
[dev-dependencies]
iced_test = "0.14"  # Match your iced version
```

The `iced_test` crate provides headless testing capabilities without requiring a display or window manager.

## Testing Update Logic

### Basic Example

Update functions are pure Rust with no dependencies - test them directly:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_decrement() {
        let mut counter = Counter { value: 0 };

        counter.update(Message::Increment);
        counter.update(Message::Increment);
        counter.update(Message::Decrement);

        assert_eq!(counter.value, 1);
    }
}
```

### Testing Complex State Changes

```rust
#[test]
fn test_file_loading() {
    let mut app = App::default();

    // Simulate file selection
    app.update(Message::FileSelected(PathBuf::from("test.pdf")));

    assert_eq!(app.current_file, Some(PathBuf::from("test.pdf")));
    assert_eq!(app.status, Status::Loading);
}

#[test]
fn test_error_handling() {
    let mut app = App::default();

    app.update(Message::LoadError("File not found".to_string()));

    assert!(matches!(app.status, Status::Error(_)));
}
```

### Testing Commands

When update returns `Command<Message>`, test the state change and verify command execution:

```rust
#[test]
fn test_save_command() {
    let mut app = App { content: "test".into(), ..Default::default() };

    let command = app.update(Message::Save);

    // Verify state change
    assert_eq!(app.status, Status::Saving);

    // Command will be executed by runtime - test its effect separately
}
```

## Headless UI Testing

### Basic Simulator Setup

```rust
use iced_test::simulator;

#[test]
fn test_button_click() {
    let mut counter = Counter { value: 0 };
    let mut ui = simulator(counter.view());

    // Click buttons by text content
    let _ = ui.click("+");
    let _ = ui.click("+");
    let _ = ui.click("-");

    // Process messages
    for message in ui.into_messages() {
        counter.update(message);
    }

    assert_eq!(counter.value, 1);
}
```

### Widget Selection

Select widgets by:
- **Text content**: `ui.click("Save")` - finds button/element containing "Save"
- **Custom selectors**: Implement custom selection logic if needed

```rust
#[test]
fn test_form_interaction() {
    let mut app = App::default();
    let mut ui = simulator(app.view());

    // Click elements by their text
    let _ = ui.click("Open File");

    for message in ui.into_messages() {
        app.update(message);
    }

    assert_eq!(app.dialog_open, true);
}
```

### Testing Multiple Interactions

```rust
#[test]
fn test_workflow() {
    let mut editor = Editor::default();
    let mut ui = simulator(editor.view());

    // Simulate user workflow
    let _ = ui.click("New Document");

    for message in ui.into_messages() {
        editor.update(message);
    }

    let mut ui = simulator(editor.view());
    let _ = ui.click("Save");

    for message in ui.into_messages() {
        editor.update(message);
    }

    assert!(editor.has_unsaved_changes == false);
}
```

## Integration Testing

Create integration tests in `tests/` directory:

```rust
// tests/integration_test.rs
use my_app::{App, Message};
use iced_test::simulator;

#[test]
fn test_complete_editing_workflow() {
    let mut app = App::new();

    // 1. Open file
    app.update(Message::FileSelected("test.pdf".into()));
    assert_eq!(app.current_file, Some("test.pdf".into()));

    // 2. Make edit
    app.update(Message::EditText("new content".into()));
    assert!(app.has_unsaved_changes);

    // 3. Save
    let mut ui = simulator(app.view());
    let _ = ui.click("Save");

    for message in ui.into_messages() {
        app.update(message);
    }

    assert!(!app.has_unsaved_changes);
}
```

## CI/CD Configuration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Run tests
        run: cargo test --all-features

      - name: Run UI tests (headless)
        run: cargo test --test '*' --features headless
        env:
          RUST_TEST_THREADS: 1  # Avoid race conditions in UI tests
```

### Local Test Running

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run specific test
cargo test test_button_click

# Run with output
cargo test -- --nocapture
```

## Common Patterns

### Pattern: Testing State Machines

```rust
#[test]
fn test_state_transitions() {
    let mut app = App { state: AppState::Idle, ..Default::default() };

    app.update(Message::StartProcess);
    assert_eq!(app.state, AppState::Processing);

    app.update(Message::ProcessComplete);
    assert_eq!(app.state, AppState::Complete);

    app.update(Message::Reset);
    assert_eq!(app.state, AppState::Idle);
}
```

### Pattern: Testing Form Validation

```rust
#[test]
fn test_form_validation() {
    let mut form = Form::default();

    form.update(Message::EmailChanged("invalid".into()));
    assert!(!form.is_valid());
    assert!(form.errors.contains(&ValidationError::InvalidEmail));

    form.update(Message::EmailChanged("valid@email.com".into()));
    assert!(form.is_valid());
    assert!(form.errors.is_empty());
}
```

### Pattern: Testing Async Operations

```rust
#[test]
fn test_async_load() {
    let mut app = App::default();

    // Start async operation
    app.update(Message::LoadData);
    assert_eq!(app.loading, true);

    // Simulate completion
    app.update(Message::DataLoaded(vec![1, 2, 3]));
    assert_eq!(app.loading, false);
    assert_eq!(app.data, vec![1, 2, 3]);
}
```

### Pattern: Snapshot Testing State

```rust
#[test]
fn test_state_snapshot() {
    let mut app = App::default();

    // Perform operations
    app.update(Message::Action1);
    app.update(Message::Action2);

    // Verify final state matches expected
    let expected = App {
        field1: "expected".into(),
        field2: 42,
        ..Default::default()
    };

    assert_eq!(app.field1, expected.field1);
    assert_eq!(app.field2, expected.field2);
}
```

### Pattern: Testing Error Recovery

```rust
#[test]
fn test_error_recovery() {
    let mut app = App::default();

    // Trigger error
    app.update(Message::LoadError("Network error".into()));
    assert!(matches!(app.state, AppState::Error(_)));

    // Recover from error
    app.update(Message::Retry);
    assert_eq!(app.state, AppState::Loading);

    // Success
    app.update(Message::LoadSuccess);
    assert_eq!(app.state, AppState::Ready);
}
```

## Best Practices

1. **Test update logic first** - Simpler and faster than UI tests
2. **Use headless tests for UI flows** - When you need to verify widget interactions
3. **Keep tests isolated** - Each test should be independent
4. **Test edge cases** - Empty states, errors, boundary conditions
5. **Use meaningful assertions** - Make failures easy to diagnose
6. **Run tests in CI** - Catch regressions early
7. **Mock external dependencies** - File I/O, network calls, etc.

## Resources

- [iced_test documentation](https://docs.iced.rs/iced_test/)
- [Iced Architecture guide](https://book.iced.rs/architecture.html)
- [Elm Architecture testing patterns](https://guide.elm-lang.org/effects/testing.html)
