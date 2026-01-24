---
name: iced-testing
description: Testing framework and patterns for iced GUI applications using both unit tests and headless UI tests with iced_test. Use when the user needs help writing tests for iced applications, setting up test infrastructure, or testing UI interactions. Covers testing update logic (business logic), headless UI testing (button clicks, interactions), integration testing (complete workflows), and CI/CD setup.
---

# Iced Testing

Test iced applications using unit tests for business logic and headless tests for UI interactions.

## Overview

Iced's Elm Architecture enables two complementary testing approaches:

1. **Unit tests** - Test update logic directly (pure functions, no UI dependencies)
2. **Headless UI tests** - Test widget interactions using `iced_test` (no display required)

Both run in CI/CD without graphics or window managers.

## Quick Start

Add `iced_test` to `Cargo.toml`:

```toml
[dev-dependencies]
iced_test = "0.14"  # Match your iced version
```

Run tests:
```bash
cargo test
```

## Testing Update Logic

Test pure state transformations without UI:

```rust
#[test]
fn test_state_change() {
    let mut app = App::default();

    app.update(Message::Increment);
    app.update(Message::Increment);

    assert_eq!(app.counter, 2);
}
```

**When to use:**
- Testing business logic
- Validating state transitions
- Testing error handling
- Fast, simple tests

**Template:** See `assets/update_logic_test.rs` for complete template with common patterns.

## Testing UI Interactions

Test widget interactions without rendering using `iced_test`:

```rust
use iced_test::simulator;

#[test]
fn test_button_click() {
    let mut app = App::default();
    let mut ui = simulator(app.view());

    // Click button by text content
    let _ = ui.click("Save");

    // Process messages
    for message in ui.into_messages() {
        app.update(message);
    }

    assert_eq!(app.saved, true);
}
```

**When to use:**
- Testing user workflows
- Verifying button clicks
- Testing conditional UI (dialogs, menus)
- Multi-step interactions

**Important:** Recreate simulator after state changes:
```rust
let mut ui = simulator(app.view());  // First interaction
let _ = ui.click("Open");
for msg in ui.into_messages() { app.update(msg); }

let mut ui = simulator(app.view());  // New simulator for next interaction
let _ = ui.click("Close");
```

**Template:** See `assets/headless_ui_test.rs` for complete template with multi-step workflows.

## Integration Testing

Test complete workflows end-to-end in `tests/` directory:

```rust
// tests/workflow_test.rs
use my_app::{App, Message};
use iced_test::simulator;

#[test]
fn test_complete_workflow() {
    let mut app = App::new();

    // Load data
    app.update(Message::LoadFile("test.pdf".into()));
    assert!(app.file_loaded);

    // Edit
    let mut ui = simulator(app.view());
    let _ = ui.click("Rotate");
    for msg in ui.into_messages() { app.update(msg); }

    // Save
    let mut ui = simulator(app.view());
    let _ = ui.click("Save");
    for msg in ui.into_messages() { app.update(msg); }

    assert!(!app.has_unsaved_changes);
}
```

**Template:** See `assets/integration_test.rs` for complete template with error recovery patterns.

## Running Tests

```bash
# All tests
cargo test

# Only unit tests
cargo test --lib

# Only integration tests
cargo test --test '*'

# Specific test
cargo test test_button_click

# With output
cargo test -- --nocapture
```

## CI/CD Setup

Headless tests work in CI without display:

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
```

## Detailed Guide

For comprehensive patterns, examples, and best practices:

**Read [references/testing-guide.md](references/testing-guide.md)** for:
- Advanced testing patterns (state machines, async, form validation)
- Error recovery testing
- Snapshot testing
- Best practices
- Troubleshooting

## Common Patterns

**Testing state machines:**
```rust
app.update(Message::Start);
assert_eq!(app.state, State::Active);
```

**Testing errors:**
```rust
app.update(Message::Error("msg".into()));
assert!(matches!(app.state, State::Error(_)));
```

**Testing workflows:**
```rust
app.update(Message::Step1);
app.update(Message::Step2);
assert!(app.complete);
```

## Resources

- **Templates:** `assets/` contains ready-to-use test templates
- **Guide:** `references/testing-guide.md` for detailed patterns
- [iced_test docs](https://docs.iced.rs/iced_test/)
- [Iced Architecture](https://book.iced.rs/architecture.html)
