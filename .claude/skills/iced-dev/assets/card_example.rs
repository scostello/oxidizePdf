//! Card Widget Example
//! Feature: `card`

use iced::widget::{button, container, scrollable, text};
use iced::{Element, Length, Task};
use iced_aw::{card, style::card as card_style, ICED_AW_FONT_BYTES};

pub fn main() -> iced::Result {
    iced::application(CardExample::new, CardExample::update, CardExample::view)
        .font(ICED_AW_FONT_BYTES)
        .run()
}

#[derive(Default)]
struct CardExample {
    card_open: bool,
}

#[derive(Debug, Clone)]
enum Message {
    OpenCard,
    CloseCard,
}

impl CardExample {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenCard => self.card_open = true,
            Message::CloseCard => self.card_open = false,
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let content: Element<Message> = if self.card_open {
            card(
                text("Card Header"),
                text("This is the card body content. Cards are useful for \
                      grouping related information with clear visual boundaries."),
            )
            .foot(text("Card Footer"))
            .style(card_style::primary)
            .on_close(Message::CloseCard)
            .max_width(400.0)
            .into()
        } else {
            button("Open Card")
                .on_press(Message::OpenCard)
                .into()
        };

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(20)
            .into()
    }
}
