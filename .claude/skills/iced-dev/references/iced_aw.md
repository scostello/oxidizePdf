# iced_aw - Additional Widgets for Iced

## Overview

`iced_aw` is the official additional widgets crate for the Iced GUI framework, providing commonly needed UI components not included in the core library. Each widget is feature-gated for minimal dependency footprint.

**Repository**: https://github.com/iced-rs/iced_aw
**Docs.rs**: https://docs.rs/iced_aw
**Maintainers**: Kaiden42, genusistimelord
**License**: MIT

## Version Compatibility

| Iced Version | iced_aw Version |
|--------------|-----------------|
| 0.14         | 0.13            |
| 0.13         | 0.11, 0.12      |

## Installation

```toml
[dependencies]
iced = "0.14.0"
iced_aw = { version = "0.13.0", features = ["card", "tabs"] }  # Select features
# OR use all widgets:
iced_aw = { version = "0.13.0", features = ["full"] }
```

## Required Font Setup

Most iced_aw widgets require loading custom fonts at application startup:

```rust
use iced_aw::ICED_AW_FONT_BYTES;

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .font(ICED_AW_FONT_BYTES)
        .run()
}
```

---

## Complete Widget Reference

### Badge (`badge`)

Visual notification indicator, commonly used for unread counts.

**Feature**: `badge`

```rust
use iced_aw::{badge, style::badge};

fn view(&self) -> Element<Message> {
    row![
        text("Messages"),
        badge(text(format!("{}", self.unread_count)))
            .style(badge::primary)
    ]
    .spacing(10)
    .into()
}
```

**Available Styles**:
- `badge::primary`, `badge::secondary`, `badge::success`
- `badge::danger`, `badge::warning`, `badge::info`
- `badge::light`, `badge::dark`

---

### Card (`card`)

Container widget for grouped content with header, body, and footer sections.

**Feature**: `card`

```rust
use iced_aw::{card, style::card};

#[derive(Debug, Clone)]
enum Message {
    OpenCard,
    CloseCard,
}

fn view(&self) -> Element<Message> {
    if self.card_open {
        card(
            text("Header Title"),                    // head
            text("Card body content goes here...")   // body
        )
        .foot(text("Footer"))
        .style(card::primary)
        .on_close(Message::CloseCard)
        .max_width(400.0)
        .into()
    } else {
        button("Open Card").on_press(Message::OpenCard).into()
    }
}
```

**Methods**:
- `.foot(impl Into<Element>)` - Add footer content
- `.on_close(Message)` - Enable close button with message
- `.max_width(f32)` - Set maximum width
- `.padding(f32)` / `.padding_head(f32)` / `.padding_body(f32)` / `.padding_foot(f32)`
- `.style(impl Fn(&Theme) -> card::Style)`

---

### Color Picker (`color_picker`)

Interactive color selection widget with HSV/RGB modes.

**Feature**: `color_picker`

```rust
use iced_aw::color_picker;
use iced::Color;

struct App {
    color: Color,
    show_picker: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ChooseColor,
    SubmitColor(Color),
    CancelColor,
}

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::ChooseColor => self.show_picker = true,
        Message::SubmitColor(color) => {
            self.color = color;
            self.show_picker = false;
        }
        Message::CancelColor => self.show_picker = false,
    }
    Task::none()
}

fn view(&self) -> Element<Message> {
    let button = button("Set Color").on_press(Message::ChooseColor);

    color_picker(
        self.show_picker,
        self.color,
        button,
        Message::CancelColor,
        Message::SubmitColor,
    )
    .into()
}
```

---

### Date Picker (`date_picker`)

Calendar-based date selection widget.

**Feature**: `date_picker`

```rust
use iced_aw::date_picker;
use iced_aw::core::date::Date;

struct App {
    date: Date,
    show_picker: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ChooseDate,
    SubmitDate(Date),
    CancelDate,
}

fn view(&self) -> Element<Message> {
    let button = button("Set Date").on_press(Message::ChooseDate);

    date_picker(
        self.show_picker,
        self.date,
        button,
        Message::CancelDate,
        Message::SubmitDate,
    )
    .into()
}
```

**Note**: Requires `chrono` for advanced date operations.

---

### Time Picker (`time_picker`)

Clock-based time selection widget.

**Feature**: `time_picker`

```rust
use iced_aw::time_picker;
use iced_aw::core::time::Time;

struct App {
    time: Time,
    show_picker: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ChooseTime,
    SubmitTime(Time),
    CancelTime,
}

fn view(&self) -> Element<Message> {
    let button = button("Set Time").on_press(Message::ChooseTime);

    time_picker(
        self.show_picker,
        self.time,
        button,
        Message::CancelTime,
        Message::SubmitTime,
    )
    .use_24h()  // Enable 24-hour format
    .into()
}
```

---

### Number Input (`number_input`)

Numeric input with increment/decrement buttons and range validation.

**Feature**: `number_input`

**Note**: Does NOT support web/WASM targets.

```rust
use iced_aw::{number_input, style::number_input};

struct App {
    value: i32,
}

#[derive(Debug, Clone)]
enum Message {
    ValueChanged(i32),
    ValueSubmitted,
}

fn view(&self) -> Element<Message> {
    number_input(&self.value, -100..=100, Message::ValueChanged)
        .step(1)                              // Increment/decrement amount
        .style(number_input::primary)
        .on_submit(Message::ValueSubmitted)   // Optional submit handler
        .into()
}
```

**Generic Types**: Works with any `num_traits::Num` type (i8, i16, i32, i64, f32, f64, etc.)

---

### Selection List (`selection_list`)

Scrollable list with single-item selection.

**Feature**: `selection_list`

```rust
use iced_aw::{SelectionList, style::selection_list::primary};

struct App {
    items: Vec<String>,
    selected: Option<usize>,
    manual_select: Option<usize>,
}

#[derive(Debug, Clone)]
enum Message {
    ItemSelected(usize, String),
}

fn view(&self) -> Element<Message> {
    SelectionList::new_with(
        &self.items[..],
        Message::ItemSelected,
        12.0,                    // Text size
        5.0,                     // Padding
        primary,                 // Style
        self.manual_select,      // Programmatic selection
        Font::default(),
    )
    .into()
}
```

---

### TabBar (`tab_bar`)

Horizontal tab navigation without content panels.

**Feature**: `tab_bar`

```rust
use iced_aw::{TabBar, TabLabel};

struct App {
    active_tab: usize,
    tabs: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
    TabClosed(usize),
}

fn view(&self) -> Element<Message> {
    let mut tab_bar = TabBar::new(Message::TabSelected);

    for (idx, label) in self.tabs.iter().enumerate() {
        tab_bar = tab_bar.push(TabLabel::Text(label.clone()));
    }

    tab_bar
        .on_close(Message::TabClosed)
        .into()
}
```

---

### Tabs (`tabs`)

Complete tabbed interface with content panels.

**Feature**: `tabs`

```rust
use iced_aw::{Tabs, TabLabel};

struct App {
    active_tab: usize,
}

#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
}

fn view(&self) -> Element<Message> {
    Tabs::new(Message::TabSelected)
        .push(
            TabLabel::Text("Tab 1".into()),
            text("Content for tab 1"),
        )
        .push(
            TabLabel::Text("Tab 2".into()),
            text("Content for tab 2"),
        )
        .set_active_tab(&self.active_tab)
        .into()
}
```

---

### Menu (`menu`)

Dropdown menu system with nested submenus.

**Feature**: `menu` (optionally `quad` for separators)

```rust
use iced_aw::{menu, menu_bar, menu_items, Menu};

fn view(&self) -> Element<Message> {
    let file_menu = Menu::new(menu_items![
        (button("New").on_press(Message::New)),
        (button("Open").on_press(Message::Open)),
        (button("Save").on_press(Message::Save)),
    ])
    .width(180.0)
    .spacing(5.0);

    let edit_menu = Menu::new(menu_items![
        (button("Cut").on_press(Message::Cut)),
        (button("Copy").on_press(Message::Copy)),
        (button("Paste").on_press(Message::Paste)),
    ])
    .width(180.0);

    menu_bar![
        (button("File"), file_menu),
        (button("Edit"), edit_menu),
    ]
    .into()
}
```

**Nested Submenus**:
```rust
let submenu = Menu::new(menu_items![
    (button("Option 1")),
    (button("Option 2")),
]);

let parent_menu = Menu::new(menu_items![
    (button("Item")),
    (submenu_button("More Options"), submenu),  // Nested!
]);
```

**Configuration**:
- `.width(f32)` - Menu width
- `.offset(f32)` - Offset from trigger
- `.spacing(f32)` - Item spacing
- `.close_on_item_click_global()` - Close when any item clicked
- `.close_on_background_click_global()` - Close on outside click

---

### Context Menu

Right-click contextual menu.

**Feature**: Part of `menu` feature

```rust
use iced_aw::ContextMenu;

fn view(&self) -> Element<Message> {
    ContextMenu::new(
        // Underlay (the clickable area)
        button("Right-click me!"),
        // Menu content generator
        || {
            column![
                button("Option 1").on_press(Message::Option1),
                button("Option 2").on_press(Message::Option2),
                button("Option 3").on_press(Message::Option3),
            ]
            .into()
        },
    )
    .into()
}
```

---

### Drop Down

Collapsible dropdown overlay.

**Feature**: Part of `menu` feature

```rust
use iced_aw::DropDown;

struct App {
    expanded: bool,
    choice: Choice,
}

#[derive(Debug, Clone)]
enum Message {
    Expand,
    Dismiss,
    Select(Choice),
}

fn view(&self) -> Element<Message> {
    let underlay = row![
        text(format!("Selected: {}", self.choice)),
        button("Expand").on_press(Message::Expand),
    ];

    let overlay = scrollable(
        column![
            button("Choice 1").on_press(Message::Select(Choice::One)),
            button("Choice 2").on_press(Message::Select(Choice::Two)),
            button("Choice 3").on_press(Message::Select(Choice::Three)),
        ]
    );

    DropDown::new(underlay, overlay, self.expanded)
        .on_dismiss(Message::Dismiss)
        .into()
}
```

---

### Sidebar (`sidebar`)

Collapsible side navigation panel.

**Feature**: `sidebar`

Includes utility layouts:
- `FlushColumn` - Column that fills available width
- `FlushRow` - Row that fills available height

```rust
use iced_aw::sidebar::{Sidebar, FlushColumn};

fn view(&self) -> Element<Message> {
    row![
        Sidebar::new(
            column![
                button("Home").on_press(Message::Navigate(Route::Home)),
                button("Settings").on_press(Message::Navigate(Route::Settings)),
            ]
        )
        .width(200.0),

        // Main content
        self.current_page_view(),
    ]
    .into()
}
```

---

### Spinner (`spinner`)

Loading indicator animation.

**Feature**: `spinner`

```rust
use iced_aw::Spinner;

fn view(&self) -> Element<Message> {
    if self.loading {
        container(Spinner::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    } else {
        // Normal content
        self.content_view()
    }
}
```

---

### Wrap (`wrap`)

Flow layout that wraps items to new lines/columns.

**Feature**: `wrap`

```rust
use iced_aw::Wrap;

fn view(&self) -> Element<Message> {
    // Horizontal wrap (default)
    Wrap::new()
        .spacing(10.0)
        .line_spacing(10.0)
        .align_items(iced::Alignment::Center)
        .push(button("Item 1"))
        .push(button("Item 2"))
        .push(button("Item 3"))
        // ... more items that wrap to new rows
        .into()
}

fn view_vertical(&self) -> Element<Message> {
    // Vertical wrap (wraps to new columns)
    Wrap::new_vertical()
        .spacing(10.0)
        .line_spacing(10.0)
        .line_minimal_length(100.0)  // Min column width before wrapping
        .push(button("Item 1"))
        .push(button("Item 2"))
        .into()
}
```

**Macros**:
```rust
use iced_aw::{wrap_horizontal, wrap_vertical};

let h = wrap_horizontal![button("A"), button("B"), button("C")];
let v = wrap_vertical![button("1"), button("2"), button("3")];
```

---

### Slide Bar (`slide_bar`)

Alternative slider control.

**Feature**: `slide_bar`

```rust
use iced_aw::SlideBar;

fn view(&self) -> Element<Message> {
    SlideBar::new(0.0..=100.0, self.value, Message::ValueChanged)
        .into()
}
```

---

### Labeled Frame

Frame container with title header.

**Feature**: Typically included by default

```rust
use iced_aw::LabeledFrame;

fn view(&self) -> Element<Message> {
    LabeledFrame::new("Settings", column![
        checkbox("Dark mode", self.dark_mode).on_toggle(Message::ToggleDarkMode),
        checkbox("Notifications", self.notifications).on_toggle(Message::ToggleNotifications),
    ])
    .into()
}
```

---

## Feature Flags Reference

| Feature | Widget(s) | Notes |
|---------|-----------|-------|
| `badge` | Badge | Notification indicators |
| `card` | Card | Content containers |
| `color_picker` | ColorPicker | HSV/RGB color selection |
| `date_picker` | DatePicker | Calendar date selection |
| `time_picker` | TimePicker | Clock time selection |
| `number_input` | NumberInput | **No web support** |
| `selection_list` | SelectionList | Scrollable selection |
| `tab_bar` | TabBar | Tab navigation only |
| `tabs` | Tabs | Full tabbed interface |
| `menu` | Menu, ContextMenu, DropDown | All menu widgets |
| `quad` | - | Menu separators |
| `sidebar` | Sidebar, FlushColumn, FlushRow | Side navigation |
| `spinner` | Spinner | Loading indicators |
| `wrap` | Wrap | Flow layouts |
| `slide_bar` | SlideBar | Slider alternative |
| `full` | All widgets | Everything included |

---

## Common Patterns

### Toggle Pattern (Picker Widgets)

All picker widgets (color, date, time) follow the same show/hide pattern:

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

fn update(&mut self, message: Message) {
    match message {
        Message::Open => self.show_picker = true,
        Message::Submit(value) => {
            self.value = value;
            self.show_picker = false;
        }
        Message::Cancel => self.show_picker = false,
    }
}

fn view(&self) -> Element<Message> {
    picker_widget(
        self.show_picker,
        self.value,
        trigger_button,
        Message::Cancel,
        Message::Submit,
    )
}
```

### Tab State Management

```rust
fn update(&mut self, message: Message) {
    match message {
        Message::TabSelected(idx) => self.active_tab = idx,
        Message::TabClosed(idx) => {
            self.tabs.remove(idx);
            // Adjust active tab to prevent out-of-bounds
            if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }
}
```

### Menu with Actions

```rust
use iced_aw::menu_items;

let menu = Menu::new(menu_items![
    (button("Action 1").on_press(Message::Action1)),
    (button("Action 2").on_press(Message::Action2)),
    (horizontal_rule(1)),  // Separator
    (button("Danger").on_press(Message::Danger).style(button::danger)),
])
.width(200.0)
.close_on_item_click_global();
```

---

## Styling

All widgets support custom styling via the `.style()` method:

```rust
use iced_aw::style::{card, badge, number_input};

// Predefined styles
card(header, body).style(card::primary)
badge(content).style(badge::danger)
number_input(&val, range, msg).style(number_input::primary)

// Custom styles
card(header, body).style(|theme| card::Style {
    border_radius: 8.0.into(),
    border_width: 2.0,
    border_color: theme.palette().primary,
    ..card::primary(theme)
})
```

---

## Resources

- **Examples**: https://github.com/iced-rs/iced_aw/tree/main/examples
- **API Docs**: https://docs.rs/iced_aw/0.13.0/iced_aw/
- **GitHub Issues**: https://github.com/iced-rs/iced_aw/issues
