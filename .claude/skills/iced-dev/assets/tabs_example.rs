//! Tabs Widget Example
//! Features: `tabs` (or `tab_bar` for navigation only)

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length, Task};
use iced_aw::{TabBar, TabLabel, ICED_AW_FONT_BYTES};

pub fn main() -> iced::Result {
    iced::application(TabExample::new, TabExample::update, TabExample::view)
        .font(ICED_AW_FONT_BYTES)
        .run()
}

struct TabExample {
    active_tab: usize,
    tabs: Vec<(String, String)>, // (label, content)
    new_tab_label: String,
    new_tab_content: String,
}

#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
    TabClosed(usize),
    NewTabLabel(String),
    NewTabContent(String),
    AddTab,
}

impl TabExample {
    fn new() -> (Self, Task<Message>) {
        let tabs = vec![
            ("Home".into(), "Welcome to the home tab!".into()),
            ("Settings".into(), "Configure your preferences here.".into()),
        ];

        (
            Self {
                active_tab: 0,
                tabs,
                new_tab_label: String::new(),
                new_tab_content: String::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(idx) => {
                self.active_tab = idx;
            }
            Message::TabClosed(idx) => {
                self.tabs.remove(idx);
                // Prevent out-of-bounds after removal
                if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                    self.active_tab = self.tabs.len() - 1;
                }
            }
            Message::NewTabLabel(label) => {
                self.new_tab_label = label;
            }
            Message::NewTabContent(content) => {
                self.new_tab_content = content;
            }
            Message::AddTab => {
                if !self.new_tab_label.is_empty() {
                    self.tabs.push((
                        std::mem::take(&mut self.new_tab_label),
                        std::mem::take(&mut self.new_tab_content),
                    ));
                    self.active_tab = self.tabs.len() - 1;
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        // Tab creation controls
        let controls = row![
            text_input("Tab label", &self.new_tab_label)
                .on_input(Message::NewTabLabel)
                .width(150),
            text_input("Tab content", &self.new_tab_content)
                .on_input(Message::NewTabContent)
                .width(200),
            button("Add Tab").on_press(Message::AddTab),
        ]
        .spacing(10);

        // Build the tab bar
        let tab_bar = self.tabs.iter().enumerate().fold(
            TabBar::new(Message::TabSelected),
            |bar, (idx, (label, _))| {
                bar.push(idx, TabLabel::Text(label.clone()))
            },
        )
        .on_close(Message::TabClosed)
        .set_active_tab(&self.active_tab);

        // Current tab content
        let content = self
            .tabs
            .get(self.active_tab)
            .map(|(_, content)| text(content).size(18))
            .unwrap_or_else(|| text("No tabs open"));

        container(
            column![controls, tab_bar, content]
                .spacing(20)
                .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
