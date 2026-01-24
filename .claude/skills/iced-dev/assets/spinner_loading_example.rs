//! Spinner Widget Example - Loading States
//! Feature: `spinner`

use iced::widget::{button, column, container, text};
use iced::{Element, Length, Task};
use iced_aw::{Spinner, ICED_AW_FONT_BYTES};
use std::time::Duration;

pub fn main() -> iced::Result {
    iced::application(
        SpinnerExample::new,
        SpinnerExample::update,
        SpinnerExample::view,
    )
    .font(ICED_AW_FONT_BYTES)
    .run()
}

struct SpinnerExample {
    state: LoadingState,
}

enum LoadingState {
    Idle,
    Loading,
    Loaded(String),
    Error(String),
}

#[derive(Debug, Clone)]
enum Message {
    StartLoading,
    LoadComplete(Result<String, String>),
}

impl SpinnerExample {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: LoadingState::Idle,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StartLoading => {
                self.state = LoadingState::Loading;
                // Simulate async operation
                Task::perform(
                    async {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        Ok::<_, String>("Data loaded successfully!".to_string())
                    },
                    Message::LoadComplete,
                )
            }
            Message::LoadComplete(result) => {
                self.state = match result {
                    Ok(data) => LoadingState::Loaded(data),
                    Err(err) => LoadingState::Error(err),
                };
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content: Element<Message> = match &self.state {
            LoadingState::Idle => {
                column![
                    text("Ready to load data").size(18),
                    button("Load Data").on_press(Message::StartLoading),
                ]
                .spacing(20)
                .into()
            }
            LoadingState::Loading => {
                column![
                    Spinner::new(),
                    text("Loading...").size(16),
                ]
                .spacing(20)
                .into()
            }
            LoadingState::Loaded(data) => {
                column![
                    text(data).size(18),
                    button("Load Again").on_press(Message::StartLoading),
                ]
                .spacing(20)
                .into()
            }
            LoadingState::Error(err) => {
                column![
                    text(format!("Error: {}", err)).size(18),
                    button("Retry").on_press(Message::StartLoading),
                ]
                .spacing(20)
                .into()
            }
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(20)
            .into()
    }
}
