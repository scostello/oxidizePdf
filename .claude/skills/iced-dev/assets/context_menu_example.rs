//! Context Menu (Right-Click Menu) Example
//! Feature: `menu`

use iced::widget::{button, column, container, text};
use iced::{Element, Length, Task};
use iced_aw::{ContextMenu, ICED_AW_FONT_BYTES};

pub fn main() -> iced::Result {
    iced::application(
        ContextMenuExample::new,
        ContextMenuExample::update,
        ContextMenuExample::view,
    )
    .font(ICED_AW_FONT_BYTES)
    .run()
}

#[derive(Default)]
struct ContextMenuExample {
    last_action: String,
    item_count: usize,
}

#[derive(Debug, Clone)]
enum Message {
    AddItem,
    RemoveItem,
    ClearAll,
    Refresh,
}

impl ContextMenuExample {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                last_action: String::new(),
                item_count: 5,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddItem => {
                self.item_count += 1;
                self.last_action = "Added item".into();
            }
            Message::RemoveItem => {
                if self.item_count > 0 {
                    self.item_count -= 1;
                }
                self.last_action = "Removed item".into();
            }
            Message::ClearAll => {
                self.item_count = 0;
                self.last_action = "Cleared all".into();
            }
            Message::Refresh => {
                self.last_action = "Refreshed".into();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        // The underlay - the area that responds to right-click
        let underlay = container(
            column![
                text("Right-click anywhere in this box").size(18),
                text(format!("Items: {}", self.item_count)).size(24),
                if !self.last_action.is_empty() {
                    text(format!("Last action: {}", self.last_action)).size(14)
                } else {
                    text("").size(14)
                },
            ]
            .spacing(10)
            .padding(20),
        )
        .width(300)
        .height(200)
        .style(container::bordered_box);

        // Wrap with context menu
        let context_menu = ContextMenu::new(underlay, || {
            // This closure generates the menu content on right-click
            column![
                button("Add Item")
                    .on_press(Message::AddItem)
                    .width(Length::Fill),
                button("Remove Item")
                    .on_press(Message::RemoveItem)
                    .width(Length::Fill),
                button("Clear All")
                    .on_press(Message::ClearAll)
                    .width(Length::Fill),
                button("Refresh")
                    .on_press(Message::Refresh)
                    .width(Length::Fill),
            ]
            .spacing(5)
            .padding(5)
            .width(150)
            .into()
        });

        container(context_menu)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
