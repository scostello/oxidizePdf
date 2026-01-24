//! NumberInput Widget Example
//! Feature: `number_input`
//! NOTE: Does NOT support web/WASM targets

use iced::widget::{column, container, row, text};
use iced::{Element, Length, Task};
use iced_aw::{number_input, style::number_input as ni_style, ICED_AW_FONT_BYTES};

pub fn main() -> iced::Result {
    iced::application(
        NumberInputExample::new,
        NumberInputExample::update,
        NumberInputExample::view,
    )
    .font(ICED_AW_FONT_BYTES)
    .run()
}

struct NumberInputExample {
    integer_value: i32,
    float_value: f64,
    bounded_value: i8,
}

#[derive(Debug, Clone)]
enum Message {
    IntegerChanged(i32),
    FloatChanged(f64),
    BoundedChanged(i8),
    Submitted,
}

impl NumberInputExample {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                integer_value: 0,
                float_value: 0.0,
                bounded_value: 50,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IntegerChanged(v) => self.integer_value = v,
            Message::FloatChanged(v) => self.float_value = v,
            Message::BoundedChanged(v) => self.bounded_value = v,
            Message::Submitted => {
                println!("Values submitted!");
                println!("  Integer: {}", self.integer_value);
                println!("  Float: {}", self.float_value);
                println!("  Bounded: {}", self.bounded_value);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        // Unbounded integer input
        let integer_input = row![
            text("Integer:").width(100),
            number_input(&self.integer_value, i32::MIN..=i32::MAX, Message::IntegerChanged)
                .step(1)
                .style(ni_style::primary),
        ]
        .spacing(10);

        // Float input with decimal step
        let float_input = row![
            text("Float:").width(100),
            number_input(&self.float_value, f64::MIN..=f64::MAX, Message::FloatChanged)
                .step(0.1)
                .style(ni_style::primary),
        ]
        .spacing(10);

        // Bounded input (useful for percentages, ratings, etc.)
        let bounded_input = row![
            text("Bounded (0-100):").width(100),
            number_input(&self.bounded_value, 0..=100, Message::BoundedChanged)
                .step(5)
                .on_submit(Message::Submitted)
                .style(ni_style::primary),
        ]
        .spacing(10);

        container(
            column![
                integer_input,
                float_input,
                bounded_input,
                text(format!(
                    "Values: int={}, float={:.2}, bounded={}",
                    self.integer_value, self.float_value, self.bounded_value
                ))
                .size(16),
            ]
            .spacing(20)
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}
