//! DatePicker Widget Example
//! Feature: `date_picker`

use iced::widget::{button, column, container, text};
use iced::{Element, Length, Task};
use iced_aw::core::date::Date;
use iced_aw::{date_picker, ICED_AW_FONT_BYTES};

pub fn main() -> iced::Result {
    iced::application(
        DatePickerExample::new,
        DatePickerExample::update,
        DatePickerExample::view,
    )
    .font(ICED_AW_FONT_BYTES)
    .run()
}

struct DatePickerExample {
    date: Date,
    show_picker: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ChooseDate,
    SubmitDate(Date),
    CancelDate,
}

impl DatePickerExample {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                date: Date::today(),
                show_picker: false,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChooseDate => {
                self.show_picker = true;
            }
            Message::SubmitDate(date) => {
                self.date = date;
                self.show_picker = false;
            }
            Message::CancelDate => {
                self.show_picker = false;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let date_button = button(text("Select Date")).on_press(Message::ChooseDate);

        let picker = date_picker(
            self.show_picker,
            self.date,
            date_button,
            Message::CancelDate,
            Message::SubmitDate,
        );

        let display = text(format!(
            "Selected: {}-{:02}-{:02}",
            self.date.year, self.date.month, self.date.day
        ))
        .size(24);

        container(column![picker, display].spacing(20).padding(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
