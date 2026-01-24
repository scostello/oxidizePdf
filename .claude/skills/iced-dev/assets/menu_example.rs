//! Menu Widget Example
//! Features: `menu`, optionally `quad` for separators

use iced::widget::{button, column, container, horizontal_rule, row, text};
use iced::{Element, Length, Task};
use iced_aw::{menu_bar, menu_items, Menu, ICED_AW_FONT_BYTES};

pub fn main() -> iced::Result {
    iced::application(MenuExample::new, MenuExample::update, MenuExample::view)
        .font(ICED_AW_FONT_BYTES)
        .run()
}

#[derive(Default)]
struct MenuExample {
    last_action: String,
}

#[derive(Debug, Clone)]
enum Message {
    // File menu
    New,
    Open,
    Save,
    SaveAs,
    Exit,
    // Edit menu
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    // View menu
    ZoomIn,
    ZoomOut,
    ZoomReset,
}

impl MenuExample {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        self.last_action = format!("{:?}", message);
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        // File menu with submenu
        let save_submenu = Menu::new(menu_items![
            (button("Save").on_press(Message::Save).width(Length::Fill)),
            (button("Save As...").on_press(Message::SaveAs).width(Length::Fill)),
        ])
        .width(150.0);

        let file_menu = Menu::new(menu_items![
            (button("New").on_press(Message::New).width(Length::Fill)),
            (button("Open").on_press(Message::Open).width(Length::Fill)),
            (row![button("Save").width(Length::Fill), text(" >")], save_submenu),
            (horizontal_rule(1)),
            (button("Exit").on_press(Message::Exit).width(Length::Fill)),
        ])
        .width(180.0)
        .offset(5.0)
        .spacing(2.0);

        // Edit menu
        let edit_menu = Menu::new(menu_items![
            (button("Undo").on_press(Message::Undo).width(Length::Fill)),
            (button("Redo").on_press(Message::Redo).width(Length::Fill)),
            (horizontal_rule(1)),
            (button("Cut").on_press(Message::Cut).width(Length::Fill)),
            (button("Copy").on_press(Message::Copy).width(Length::Fill)),
            (button("Paste").on_press(Message::Paste).width(Length::Fill)),
        ])
        .width(180.0)
        .offset(5.0);

        // View menu
        let view_menu = Menu::new(menu_items![
            (button("Zoom In").on_press(Message::ZoomIn).width(Length::Fill)),
            (button("Zoom Out").on_press(Message::ZoomOut).width(Length::Fill)),
            (button("Reset Zoom").on_press(Message::ZoomReset).width(Length::Fill)),
        ])
        .width(180.0)
        .offset(5.0);

        // Menu bar
        let menu_bar = menu_bar![
            (button("File"), file_menu),
            (button("Edit"), edit_menu),
            (button("View"), view_menu),
        ];

        // Main content
        let content = container(
            text(if self.last_action.is_empty() {
                "Click a menu item to see the action here"
            } else {
                &self.last_action
            })
            .size(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

        column![menu_bar, content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
