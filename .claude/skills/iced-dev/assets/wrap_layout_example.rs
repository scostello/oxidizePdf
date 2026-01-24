//! Wrap Widget Example - Flow Layouts
//! Feature: `wrap`
//!
//! The Wrap widget creates responsive layouts where items
//! automatically wrap to new rows (horizontal) or columns (vertical)
//! when they exceed the available space.

use iced::widget::{button, column, container, row, slider, text};
use iced::{Alignment, Element, Length, Task};
use iced_aw::{Wrap, ICED_AW_FONT_BYTES};

pub fn main() -> iced::Result {
    iced::application(WrapExample::new, WrapExample::update, WrapExample::view)
        .font(ICED_AW_FONT_BYTES)
        .run()
}

struct WrapExample {
    spacing: f32,
    line_spacing: f32,
    alignment: Alignment,
}

#[derive(Debug, Clone)]
enum Message {
    SpacingChanged(f32),
    LineSpacingChanged(f32),
    AlignStart,
    AlignCenter,
    AlignEnd,
}

impl WrapExample {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                spacing: 10.0,
                line_spacing: 10.0,
                alignment: Alignment::Start,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SpacingChanged(v) => self.spacing = v,
            Message::LineSpacingChanged(v) => self.line_spacing = v,
            Message::AlignStart => self.alignment = Alignment::Start,
            Message::AlignCenter => self.alignment = Alignment::Center,
            Message::AlignEnd => self.alignment = Alignment::End,
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        // Controls
        let controls = column![
            row![
                text("Spacing:"),
                slider(0.0..=50.0, self.spacing, Message::SpacingChanged),
                text(format!("{:.0}", self.spacing)),
            ]
            .spacing(10),
            row![
                text("Line Spacing:"),
                slider(0.0..=50.0, self.line_spacing, Message::LineSpacingChanged),
                text(format!("{:.0}", self.line_spacing)),
            ]
            .spacing(10),
            row![
                text("Alignment:"),
                button("Start").on_press(Message::AlignStart),
                button("Center").on_press(Message::AlignCenter),
                button("End").on_press(Message::AlignEnd),
            ]
            .spacing(10),
        ]
        .spacing(10);

        // Create items of varying sizes
        let items: Vec<Element<Message>> = (1..=15)
            .map(|i| {
                let width = 60 + (i % 3) * 30; // Varying widths: 60, 90, 120
                button(text(format!("Item {}", i)))
                    .width(width)
                    .into()
            })
            .collect();

        // Horizontal wrap (items wrap to new rows)
        let mut horizontal_wrap = Wrap::new()
            .spacing(self.spacing)
            .line_spacing(self.line_spacing)
            .align_items(self.alignment);

        for item in items {
            horizontal_wrap = horizontal_wrap.push(item);
        }

        // Vertical wrap example (items wrap to new columns)
        let vertical_items: Vec<Element<Message>> = (1..=8)
            .map(|i| button(text(format!("V{}", i))).into())
            .collect();

        let mut vertical_wrap = Wrap::new_vertical()
            .spacing(self.spacing)
            .line_spacing(self.line_spacing)
            .align_items(self.alignment);

        for item in vertical_items {
            vertical_wrap = vertical_wrap.push(item);
        }

        container(
            column![
                controls,
                text("Horizontal Wrap (items wrap to new rows):").size(16),
                container(horizontal_wrap)
                    .width(Length::Fill)
                    .style(container::bordered_box)
                    .padding(10),
                text("Vertical Wrap (items wrap to new columns):").size(16),
                container(vertical_wrap)
                    .height(200)
                    .width(Length::Fill)
                    .style(container::bordered_box)
                    .padding(10),
            ]
            .spacing(20)
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
