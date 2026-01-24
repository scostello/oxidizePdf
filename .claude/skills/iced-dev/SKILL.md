---
name: iced-dev
description: Expert guidance for building cross-platform GUI applications with the iced Rust framework, covering architecture, widgets, styling, async operations, and best practices
context: fork
---

# Iced GUI Development Skill

## Overview

Iced is a cross-platform GUI library for Rust focused on simplicity and type-safety. It's inspired by The Elm Architecture and designed for advanced Rust programmers who want to build reactive, type-safe graphical user interfaces.

**Current Version**: 0.14.0
**Status**: Experimental software (as of 2025)
**Philosophy**: Leverages Rust's type system to minimize runtime errors
**Platforms**: Windows, macOS, Linux, Web (via WebAssembly)

## What's New in 0.14.0

- **Reactive Rendering**: Only redraws when necessary for improved performance
- **Animation API**: Built-in support for smooth transitions and animated values
- **Time Travel Debugging**: Step through application state history
- **Hot Reloading**: Rapid development iteration
- **New Widgets**: `table`, `grid`, `sensor`, `float`, and `pin`
- **Task Improvements**: `Task::sip()` for progress streaming, `.abortable()` for cancellation
- **Simplified Entry Points**: `iced::run()` for simple apps, `iced::application()` builder for advanced configuration

## Core Architecture (The Elm Architecture)

Every iced application is built around four fundamental concepts:

### 1. State
The application's current state, typically a custom struct:

```rust
struct Counter {
    value: i64,
}
```

### 2. Messages
Enum representing all possible user interactions or events:

```rust
#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    Reset,
}
```

### 3. Update Function
Handles messages and modifies state. Can return `Task` for async operations:

```rust
fn update(state: &mut Counter, message: Message) -> Task<Message> {
    match message {
        Message::Increment => state.value += 1,
        Message::Decrement => state.value -= 1,
        Message::Reset => state.value = 0,
    }
    Task::none()
}
```

### 4. View Function
Renders the current state as interactive widgets:

```rust
fn view(state: &Counter) -> Element<'_, Message> {
    column![
        button("+").on_press(Message::Increment),
        text(state.value),
        button("-").on_press(Message::Decrement),
    ].into()
}
```

## Application Entry Point

### Simple Application (iced::run)
For basic applications with minimal configuration:

```rust
use iced::widget::{button, column, text};

pub fn main() -> iced::Result {
    iced::run(update, view)
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

fn update(counter: &mut u64, message: Message) {
    match message {
        Message::Increment => *counter += 1,
    }
}

fn view(counter: &u64) -> iced::Element<'_, Message> {
    column![
        text(counter).size(50),
        button("+").on_press(Message::Increment),
    ]
    .into()
}
```

### Advanced Application (iced::application)
For applications needing initialization, subscriptions, themes, or window configuration:

```rust
use iced::widget::{column, button, text};
use iced::{Element, Theme, Task, Subscription};

pub fn main() -> iced::Result {
    iced::application(Counter::new, Counter::update, Counter::view)
        .theme(Counter::theme)
        .subscription(Counter::subscription)
        .centered()
        .run()
}

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

impl Counter {
    fn new() -> (Self, Task<Message>) {
        (Self { value: 0 }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => self.value += 1,
            Message::Decrement => self.value -= 1,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            button("+").on_press(Message::Increment),
            text(self.value).size(50),
            button("-").on_press(Message::Decrement)
        ]
        .padding(20)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
```

### Window Configuration
Configure window properties using the builder pattern:

```rust
use iced::window;
use iced::{Size, Settings};

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .settings(Settings {
            window: window::Settings {
                size: Size::new(800.0, 600.0),
                position: window::Position::Centered,
                min_size: Some(Size::new(400.0, 300.0)),
                resizable: true,
                decorations: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}
```

## Widgets and Layout

### Built-in Widgets
- **Text**: `text("content")`, `text(variable)`
- **Button**: `button("Click me").on_press(Message::Clicked)`
- **Text Input**: `text_input("placeholder", &value).on_input(Message::InputChanged)`
- **Checkbox**: `checkbox("Label", is_checked).on_toggle(Message::Toggled)`
- **Radio**: `radio("Option", value, selected, Message::Selected)`
- **Slider**: `slider(0..=100, value, Message::Changed)`
- **Progress Bar**: `progress_bar(0.0..=100.0, value)`
- **Scrollable**: `scrollable(content)`
- **Container**: `container(widget).padding(20).center_x()`
- **Column**: `column![widget1, widget2, widget3]`
- **Row**: `row![widget1, widget2, widget3]`

### New Widgets in 0.14

#### Table Widget
Display tabular data with column headers, sorting, and scrolling:

```rust
use iced::widget::{table, text};

#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

fn view(users: &[User]) -> Element<'_, Message> {
    let columns = vec![
        table::Column::new("ID").width(50),
        table::Column::new("Name").width(150),
        table::Column::new("Email").width(200),
    ];

    table(columns, users, |user| {
        vec![
            text(user.id).into(),
            text(&user.name).into(),
            text(&user.email).into(),
        ]
    })
    .into()
}
```

#### Grid Widget
Create two-dimensional layouts with automatic sizing:

```rust
use iced::widget::{grid, button, text};

fn view() -> Element<'static, Message> {
    grid![
        text("Row 1, Col 1"), text("Row 1, Col 2"), text("Row 1, Col 3"),
        text("Row 2, Col 1"), button("Click"), text("Row 2, Col 3"),
        text("Row 3, Col 1"), text("Row 3, Col 2"), text("Row 3, Col 3"),
    ]
    .spacing(10)
    .padding(20)
    .into()
}
```

#### Float Widget
Position content as overlays without affecting main layout:

```rust
use iced::widget::{button, container, float, text};
use iced::{Element, Fill};

fn view() -> Element<'static, Message> {
    float(
        container("Main Content")
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill),
        button("Floating Button")
            .on_press(Message::FloatingClicked)
    )
    .into()
}
```

#### Sensor Widget
Measure the size of content and respond to changes:

```rust
use iced::widget::{column, sensor, text};
use iced::{Element, Size};

struct App {
    content_size: Option<Size>,
}

#[derive(Debug, Clone)]
enum Message {
    SizeChanged(Size),
}

fn view(app: &App) -> Element<'_, Message> {
    column![
        sensor(text("This text is being measured"))
            .on_resize(Message::SizeChanged),
        if let Some(size) = app.content_size {
            text!("Size: {:.1} x {:.1}", size.width, size.height)
        } else {
            text("Measuring...")
        }
    ]
    .spacing(10)
    .into()
}
```

### Layout System

**Length Types**:
- `Length::Fill` - Takes available space
- `Length::Shrink` - Minimal space needed
- `Length::Fixed(pixels)` - Fixed pixel size

**Spacing and Padding**:
```rust
column![
    text("Title"),
    text("Subtitle"),
]
.spacing(10)        // Space between widgets
.padding(20)        // Inner padding
.width(Length::Fill)
.height(Length::Shrink)
```

### Builder Pattern
All widgets use builder pattern for configuration:

```rust
button(text("Submit"))
    .on_press(Message::Submit)
    .padding(10)
    .style(button::primary)
    .width(Length::Fill)
```

## Tasks (Async Operations)

Tasks represent concurrent operations that produce messages:

### Common Task Operations

```rust
// No operation
Task::none()

// Execute future
Task::perform(async_operation(), Message::Completed)

// Multiple tasks in parallel
Task::batch([task1, task2, task3])

// Chain tasks sequentially
task1.then(|_| task2)

// Map task result
task.map(|result| Message::Transform(result))
```

### Example: HTTP Request
```rust
use iced::{Task, Element};
use iced::widget::{button, column, text};

struct App {
    data: Option<String>,
    loading: bool,
}

#[derive(Debug, Clone)]
enum Message {
    FetchData,
    DataFetched(String),
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::FetchData => {
            app.loading = true;
            Task::perform(fetch_from_api(), Message::DataFetched)
        }
        Message::DataFetched(data) => {
            app.loading = false;
            app.data = Some(data);
            Task::none()
        }
    }
}

async fn fetch_from_api() -> String {
    // API call implementation
    "API Response Data".to_string()
}
```

### Progress Tracking with Task::sip (New in 0.14)

Stream progress updates during long-running operations:

```rust
use iced::{Task, task};
use iced::widget::{button, progress_bar, column, text};

struct Download {
    progress: f32,
    state: State,
}

enum State {
    Idle,
    Downloading { _handle: task::Handle },
    Finished,
}

#[derive(Debug, Clone)]
enum Message {
    Start,
    Progress(f32),
    Finished(Result<(), String>),
}

impl Download {
    fn start(&mut self) -> Task<Message> {
        let (task, handle) = Task::sip(
            download_file("https://example.com/large-file.zip"),
            Message::Progress,
            Message::Finished,
        )
        .abortable();

        self.state = State::Downloading {
            _handle: handle.abort_on_drop(),
        };
        self.progress = 0.0;

        task
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Idle => {
                button("Start Download")
                    .on_press(Message::Start)
                    .into()
            }
            State::Downloading { .. } => {
                column![
                    text!("Downloading: {:.1}%", self.progress),
                    progress_bar(0.0..=100.0, self.progress)
                ]
                .spacing(10)
                .into()
            }
            State::Finished => {
                text("Download Complete!").into()
            }
        }
    }
}
```

### Composing Tasks

Chain and batch tasks for complex workflows:

```rust
fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Initialize => {
            // Execute multiple tasks in parallel
            Task::batch([
                Task::perform(load_user_data(), Message::UserDataLoaded),
                Task::perform(load_settings(), Message::SettingsLoaded),
                Task::perform(check_updates(), Message::UpdatesChecked),
            ])
        }
        Message::SaveAndExit => {
            // Chain tasks sequentially
            Task::perform(save_state(state.clone()), Message::StateSaved)
                .then(|_| iced::exit())
        }
        _ => Task::none(),
    }
}
```

## Animation API (New in 0.14)

Create smooth visual transitions with built-in easing functions:

```rust
use iced::widget::{button, column, container, text};
use iced::{Element, Fill, animation, Animation, Task};
use std::time::Duration;

struct App {
    position: f32,
    animation: Animation<f32>,
}

#[derive(Debug, Clone)]
enum Message {
    Animate,
    AnimationStep(f32),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                position: 0.0,
                animation: Animation::new(Duration::from_millis(500)),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Animate => {
                self.animation = Animation::new(Duration::from_millis(500))
                    .with_easing(animation::Easing::EaseInOut);

                animation::stream(self.animation.clone(), 0.0, 100.0)
                    .map(Message::AnimationStep)
            }
            Message::AnimationStep(value) => {
                self.position = value;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            container("Animated Element")
                .width(100)
                .height(100)
                .style(move |theme| {
                    container::Style::default()
                        .with_border_radius(self.position)
                }),
            button("Animate").on_press(Message::Animate)
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}
```

Available easing functions:
- `Easing::Linear`
- `Easing::EaseIn`
- `Easing::EaseOut`
- `Easing::EaseInOut`

## Subscriptions

Declarative way to listen to external events:

### Time-based Subscriptions
```rust
use iced::time;

fn subscription(&self) -> Subscription<Message> {
    time::every(Duration::from_secs(1))
        .map(|_| Message::Tick)
}
```

### Keyboard Events
```rust
use iced::keyboard;

fn subscription(&self) -> Subscription<Message> {
    keyboard::on_key_press(|key, modifiers| {
        match key {
            keyboard::Key::Character("q") => Some(Message::Quit),
            _ => None,
        }
    })
}
```

### Multiple Subscriptions
```rust
fn subscription(&self) -> Subscription<Message> {
    Subscription::batch(vec![
        time::every(Duration::from_secs(1)).map(|_| Message::Tick),
        keyboard::on_key_press(handle_key),
    ])
}
```

## Theming and Styling

### Built-in Themes
```rust
fn theme(&self) -> Theme {
    if self.dark_mode {
        Theme::Dark
    } else {
        Theme::Light
    }
}
```

### Custom Styling
Widgets accept styling functions:

```rust
button(text("Custom"))
    .style(|theme, status| {
        button::Style {
            background: Some(Color::from_rgb(0.2, 0.5, 0.8).into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            text_color: Color::WHITE,
            ..button::primary(theme, status)
        }
    })
```

## Best Practices

### 1. State Management
- Keep state minimal and derived values in view
- Use type-safe enums for application screens
- Consider state machines for complex flows

```rust
enum Screen {
    Loading,
    Loaded { data: Vec<Item> },
    Error(String),
}
```

### 2. Message Organization
- Group related messages in nested enums
- Use descriptive message names
- Avoid generic messages like `Update`

```rust
enum Message {
    User(UserMessage),
    Network(NetworkMessage),
    Ui(UiMessage),
}
```

### 3. Component Composition
- Break complex views into functions
- Create reusable view components
- Pass message constructors for composability

```rust
fn user_card<'a>(user: &User, on_click: Message) -> Element<'a, Message> {
    button(
        column![
            text(&user.name),
            text(&user.email),
        ]
    )
    .on_press(on_click)
    .into()
}
```

### 4. Performance
- Use `Lazy` widget for expensive views
- Avoid cloning large data in view
- Consider diffing for large lists

### 5. Error Handling
- Return `Task` for fallible operations
- Use `Result` in messages for error propagation
- Provide user feedback for failures

```rust
enum Message {
    LoadData,
    DataLoaded(Result<Data, Error>),
}
```

## Common Patterns

### Loading State Pattern
```rust
enum State {
    Idle,
    Loading,
    Loaded(Data),
    Error(String),
}

fn view(&self) -> Element<Message> {
    match &self.state {
        State::Idle => button("Load").on_press(Message::Load).into(),
        State::Loading => text("Loading...").into(),
        State::Loaded(data) => show_data(data),
        State::Error(err) => text(err).into(),
    }
}
```

### Form Pattern
```rust
struct Form {
    name: String,
    email: String,
    validation_error: Option<String>,
}

enum Message {
    NameChanged(String),
    EmailChanged(String),
    Submit,
}

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::NameChanged(value) => {
            self.form.name = value;
            Task::none()
        }
        Message::Submit => {
            if self.form.name.is_empty() {
                self.form.validation_error = Some("Name required".into());
                Task::none()
            } else {
                Task::perform(submit_form(self.form.clone()), Message::Submitted)
            }
        }
    }
}
```

### Navigation Pattern
```rust
enum Route {
    Home,
    Settings,
    Profile(UserId),
}

fn view(&self) -> Element<Message> {
    match self.current_route {
        Route::Home => home_view(),
        Route::Settings => settings_view(),
        Route::Profile(id) => profile_view(id),
    }
}
```

## Debug Tools

### Debug Overlay

Iced includes built-in debug overlay accessible by pressing F12:

```rust
use iced::window;

fn subscription(&self) -> Subscription<Message> {
    window::debug_toggle()
}
```

Debug view features:
- FPS counter
- Frame time graph
- Widget layout boundaries
- Performance metrics

### Time Travel Debugging (New in 0.14)

Step through application state history to diagnose issues and understand state transitions. This feature allows you to move backward and forward through message history to inspect how your application state changes over time.

### Hot Reloading (New in 0.14)

Rapid development iteration with hot reloading support. Changes to your code can be reflected in the running application without losing state, significantly speeding up the development process.

## Integration with oxidize-pdf

When building PDF viewers or editors with iced 0.14:

### 1. Rendering PDFs
- Use `Image` widget with decoded PDF pages
- Consider `Scrollable` for multi-page documents
- Use `Canvas` for custom PDF rendering
- **NEW**: Use `sensor` widget to measure page dimensions for responsive layouts
- **NEW**: Use `float` widget for overlay UI (toolbars, annotations)

### 2. Performance
- Render pages lazily (on-demand)
- Cache rendered pages as images
- Use background tasks with `Task::perform()` for page rendering
- **NEW**: Use `Task::sip()` for streaming page rendering progress
- **NEW**: Leverage reactive rendering to minimize redraws

### 3. Interaction
- Track mouse position for text selection
- Use `Scrollable` for panning
- Implement zoom with transform matrix
- **NEW**: Use Animation API for smooth zoom transitions
- **NEW**: Use `grid` for thumbnail views

### Example Integration (0.14)
```rust
use iced::{Task, task};
use iced::widget::{scrollable, column, image, float, button, progress_bar};

struct PdfViewer {
    rendered_pages: HashMap<usize, ImageHandle>,
    render_progress: f32,
    render_state: RenderState,
}

enum RenderState {
    Idle,
    Rendering { _handle: task::Handle },
    Complete,
}

enum Message {
    RenderPage(usize),
    RenderProgress(f32),
    PageRendered(Result<(usize, ImageHandle), String>),
    ZoomIn,
    ZoomOut,
}

fn update(viewer: &mut PdfViewer, message: Message) -> Task<Message> {
    match message {
        Message::RenderPage(page_num) => {
            let (task, handle) = Task::sip(
                render_pdf_page(page_num),
                Message::RenderProgress,
                Message::PageRendered,
            )
            .abortable();

            viewer.render_state = RenderState::Rendering {
                _handle: handle.abort_on_drop(),
            };
            viewer.render_progress = 0.0;

            task
        }
        Message::RenderProgress(progress) => {
            viewer.render_progress = progress;
            Task::none()
        }
        Message::PageRendered(Ok((page_num, handle))) => {
            viewer.rendered_pages.insert(page_num, handle);
            viewer.render_state = RenderState::Complete;
            Task::none()
        }
        Message::ZoomIn | Message::ZoomOut => {
            // Use Animation API for smooth zoom
            Task::none()
        }
        _ => Task::none(),
    }
}

fn view(viewer: &PdfViewer) -> Element<Message> {
    let pages = column(
        viewer.rendered_pages
            .iter()
            .map(|(_, handle)| image(handle.clone()).into())
            .collect()
    );

    let content = scrollable(pages);

    // Floating toolbar overlay
    float(
        content,
        row![
            button("Zoom In").on_press(Message::ZoomIn),
            button("Zoom Out").on_press(Message::ZoomOut),
        ]
        .spacing(10)
    )
    .into()
}

async fn render_pdf_page(page_num: usize) -> Result<(usize, ImageHandle), String> {
    // PDF rendering implementation
    Ok((page_num, ImageHandle::default()))
}
```

### Advanced Features with 0.14

**Thumbnail Grid View**:
```rust
use iced::widget::grid;

fn thumbnail_view(pages: &[ImageHandle]) -> Element<Message> {
    let thumbnails: Vec<Element<Message>> = pages
        .iter()
        .map(|handle| image(handle.clone()).into())
        .collect();

    grid![thumbnails].spacing(10).into()
}
```

**Smooth Zoom Transitions**:
```rust
use iced::animation::{self, Animation};

struct ZoomState {
    current: f32,
    animation: Animation<f32>,
}

fn animate_zoom(from: f32, to: f32) -> Task<Message> {
    let animation = Animation::new(Duration::from_millis(300))
        .with_easing(animation::Easing::EaseInOut);

    animation::stream(animation, from, to)
        .map(Message::ZoomChanged)
}
```

## Resources

- **Official Docs**: https://docs.rs/iced/0.14.0/iced/
- **Book**: https://book.iced.rs
- **GitHub**: https://github.com/iced-rs/iced
- **Examples**: https://github.com/iced-rs/iced/tree/0.14.0/examples
- **Changelog**: https://github.com/iced-rs/iced/releases/tag/0.14.0

## Troubleshooting

### Common Issues

**Lifetime Errors in View**
- Solution: Use `Element<'a, Message>` and ensure borrowed data outlives view

**Message Not Handling**
- Solution: Check `update` has exhaustive match on all messages

**Layout Not Responsive**
- Solution: Use `Length::Fill` and avoid fixed sizes

**Async Operation Blocking UI**
- Solution: Move long operations to `Task::perform`

**Heavy Redraws**
- Solution: Use `Lazy` widget or split state to minimize updates

---

## Version-Specific Notes (0.14.0)

### API Changes from Previous Versions
- Use `iced::run()` for simple applications instead of the full `Application` trait
- Use `iced::application()` builder pattern for advanced configuration
- `new()` function returns `(Self, Task<Message>)` for initialization
- Window settings configured via builder methods, not `Settings` struct in most cases
- Reactive rendering improves performance by only redrawing when state changes

### New Capabilities
- Use `Task::sip()` for progress streaming in long-running operations
- Use `.abortable()` to create cancellable tasks with `task::Handle`
- Use the Animation API for smooth transitions instead of manual interpolation
- Leverage new widgets (`table`, `grid`, `float`, `sensor`) for complex layouts
- Take advantage of time travel debugging for state inspection

### Best Practices for 0.14
- Prefer `iced::run()` for simple apps to reduce boilerplate
- Use `animation::stream()` for smooth UI transitions
- Implement progress tracking with `Task::sip()` for better UX
- Use `sensor` widget for responsive layouts based on content size
- Use `float` widget for tooltips, dropdowns, and overlays
- Leverage reactive rendering by structuring state changes efficiently

---

## iced_aw - Additional Widgets Library

For widgets beyond the core iced library, use `iced_aw` - the official additional widgets crate.

**Full documentation**: See `references/iced_aw.md`
**Example code**: See `assets/` directory

### Installation

```toml
[dependencies]
iced = "0.14.0"
iced_aw = { version = "0.13.0", features = ["card", "tabs", "menu"] }
# Or use all widgets:
iced_aw = { version = "0.13.0", features = ["full"] }
```

### Required Font Setup

```rust
use iced_aw::ICED_AW_FONT_BYTES;

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .font(ICED_AW_FONT_BYTES)  // Required for iced_aw widgets
        .run()
}
```

### Available Widgets Quick Reference

| Widget | Feature | Use Case |
|--------|---------|----------|
| **Badge** | `badge` | Notification counts, status indicators |
| **Card** | `card` | Content containers with header/body/footer |
| **ColorPicker** | `color_picker` | HSV/RGB color selection |
| **DatePicker** | `date_picker` | Calendar date selection |
| **TimePicker** | `time_picker` | Clock time selection |
| **NumberInput** | `number_input` | Numeric input with +/- buttons (**no web**) |
| **SelectionList** | `selection_list` | Scrollable single-selection list |
| **TabBar** | `tab_bar` | Tab navigation (no content panels) |
| **Tabs** | `tabs` | Full tabbed interface with content |
| **Menu** | `menu` | Dropdown menus with nesting |
| **ContextMenu** | `menu` | Right-click menus |
| **DropDown** | `menu` | Collapsible dropdown overlay |
| **Sidebar** | `sidebar` | Side navigation panel |
| **Spinner** | `spinner` | Loading indicator |
| **Wrap** | `wrap` | Flow layout (items wrap to new lines) |
| **SlideBar** | `slide_bar` | Alternative slider |
| **LabeledFrame** | - | Frame with title |

### Common Patterns

**Picker Pattern** (ColorPicker, DatePicker, TimePicker):
```rust
struct App {
    show_picker: bool,
    value: T,  // Color, Date, or Time
}

enum Message {
    Open,
    Submit(T),
    Cancel,
}

fn view(&self) -> Element<Message> {
    picker_widget(
        self.show_picker,
        self.value,
        button("Select"),
        Message::Cancel,
        Message::Submit,
    )
}
```

**Menu Pattern**:
```rust
use iced_aw::{menu_bar, menu_items, Menu};

let file_menu = Menu::new(menu_items![
    (button("New").on_press(Message::New)),
    (button("Open").on_press(Message::Open)),
])
.width(180.0);

menu_bar![(button("File"), file_menu)]
```

**Tab Management**:
```rust
fn update(&mut self, message: Message) {
    match message {
        Message::TabSelected(idx) => self.active_tab = idx,
        Message::TabClosed(idx) => {
            self.tabs.remove(idx);
            if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }
}
```

### Example Files

| File | Widget | Description |
|------|--------|-------------|
| `assets/card_example.rs` | Card | Open/close card with styling |
| `assets/tabs_example.rs` | TabBar | Dynamic tab creation/closure |
| `assets/date_picker_example.rs` | DatePicker | Calendar selection |
| `assets/menu_example.rs` | Menu | Nested dropdown menus |
| `assets/number_input_example.rs` | NumberInput | Bounded numeric input |
| `assets/context_menu_example.rs` | ContextMenu | Right-click menu |
| `assets/spinner_loading_example.rs` | Spinner | Loading state pattern |
| `assets/wrap_layout_example.rs` | Wrap | Responsive flow layouts |

### Integration with PDF Viewer

For PDF viewer applications, these iced_aw widgets are particularly useful:

```rust
// Tab-based document management
let document_tabs = Tabs::new(Message::TabSelected)
    .push(TabLabel::Text("document.pdf".into()), page_view)
    .push(TabLabel::Text("other.pdf".into()), other_view)
    .on_close(Message::CloseDocument);

// Context menu for page operations
let page_context = ContextMenu::new(page_container, || {
    column![
        button("Copy Selection"),
        button("Add Bookmark"),
        button("Rotate Page"),
    ].into()
});

// Sidebar for navigation
let sidebar = Sidebar::new(column![
    button("Thumbnails"),
    button("Bookmarks"),
    button("Annotations"),
]);
```

---

When helping with iced development:
1. Always follow The Elm Architecture pattern
2. Emphasize type safety and exhaustive pattern matching
3. Suggest tasks for async operations (use `Task::sip()` for progress tracking)
4. Recommend subscriptions for external events
5. Guide toward composable, reusable components
6. Consider performance implications and leverage reactive rendering in 0.14
7. Use new 0.14 widgets (`table`, `grid`, `float`, `sensor`) when appropriate
8. Suggest the Animation API for smooth transitions instead of manual timing
9. Recommend `iced::run()` for simple apps, `iced::application()` for complex ones
10. **Use iced_aw widgets for common UI patterns** (tabs, menus, pickers, cards)
11. **Remember iced_aw requires font loading** via `ICED_AW_FONT_BYTES`
12. **Note NumberInput doesn't support web/WASM targets**
