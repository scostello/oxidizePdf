use iced::{
    widget::{button, column, container, horizontal_space, image as img, row, scrollable, text},
    Element, Length, Task, Theme,
};
use std::path::PathBuf;

mod pdf_viewer;
mod renderer;
mod viewport;

use pdf_viewer::PdfDocument;
use viewport::Viewport;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter("oxidize_pdf_editor=debug,info")
        .init();

    iced::application("PDF Editor", PdfEditor::update, PdfEditor::view)
        .theme(|_| Theme::Dark)
        .run_with(PdfEditor::new)
}

#[derive(Debug, Clone)]
enum Message {
    OpenFile,
    FileOpened(Result<PdfDocument, String>),
    PageChanged(usize),
    ZoomIn,
    ZoomOut,
    ZoomReset,
    Pan(f32, f32),
    CloseTab(usize),
    SelectTab(usize),
}

struct Tab {
    document: PdfDocument,
    viewport: Viewport,
}

struct PdfEditor {
    tabs: Vec<Tab>,
    active_tab: usize,
}

impl PdfEditor {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                tabs: Vec::new(),
                active_tab: 0,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile => {
                return Task::perform(
                    async {
                        // For now, use a file dialog (we'll implement this next)
                        // Hardcoded for testing
                        let path = PathBuf::from("test.pdf");
                        PdfDocument::load(path).await
                    },
                    Message::FileOpened,
                );
            }
            Message::FileOpened(result) => {
                match result {
                    Ok(document) => {
                        let viewport = Viewport::new(document.page_count());
                        self.tabs.push(Tab { document, viewport });
                        self.active_tab = self.tabs.len() - 1;
                    }
                    Err(e) => {
                        tracing::error!("Failed to open PDF: {}", e);
                    }
                }
            }
            Message::PageChanged(page) => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.viewport.set_page(page);
                }
            }
            Message::ZoomIn => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.viewport.zoom_in();
                }
            }
            Message::ZoomOut => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.viewport.zoom_out();
                }
            }
            Message::ZoomReset => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.viewport.reset_zoom();
                }
            }
            Message::Pan(dx, dy) => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.viewport.pan(dx, dy);
                }
            }
            Message::CloseTab(index) => {
                if index < self.tabs.len() {
                    self.tabs.remove(index);
                    if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                        self.active_tab = self.tabs.len() - 1;
                    }
                }
            }
            Message::SelectTab(index) => {
                if index < self.tabs.len() {
                    self.active_tab = index;
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let content = if self.tabs.is_empty() {
            // Welcome screen
            container(
                column![
                    text("PDF Editor").size(32),
                    text("Open a PDF document to get started").size(16),
                    button("Open PDF").on_press(Message::OpenFile)
                ]
                .spacing(20)
                .align_x(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else {
            // Tab bar
            let mut tab_bar = row![].spacing(5);
            for (idx, tab) in self.tabs.iter().enumerate() {
                let tab_button = button(
                    row![
                        text(tab.document.file_name()).size(14),
                        button("×")
                            .on_press(Message::CloseTab(idx))
                    ]
                    .spacing(5),
                )
                .on_press(Message::SelectTab(idx));
                tab_bar = tab_bar.push(tab_button);
            }
            tab_bar = tab_bar.push(button("+").on_press(Message::OpenFile));

            let main_content = if let Some(tab) = self.tabs.get(self.active_tab) {
                let toolbar = row![
                    button("Open").on_press(Message::OpenFile),
                    horizontal_space(),
                    button("−").on_press(Message::ZoomOut),
                    text(format!("{}%", (tab.viewport.zoom() * 100.0) as i32)),
                    button("+").on_press(Message::ZoomIn),
                    button("Reset").on_press(Message::ZoomReset),
                    horizontal_space(),
                    text(format!(
                        "Page {} of {}",
                        tab.viewport.current_page() + 1,
                        tab.document.page_count()
                    )),
                    button("◀")
                        .on_press_maybe(
                            (tab.viewport.current_page() > 0)
                                .then(|| Message::PageChanged(tab.viewport.current_page() - 1))
                        ),
                    button("▶")
                        .on_press_maybe(
                            (tab.viewport.current_page() + 1 < tab.document.page_count())
                                .then(|| Message::PageChanged(tab.viewport.current_page() + 1))
                        ),
                ]
                .spacing(10)
                .padding(10);

                // Render current page
                let page_view = if let Some(rendered) = tab.document.get_rendered_page(
                    tab.viewport.current_page(),
                    tab.viewport.zoom(),
                ) {
                    scrollable(container(img(rendered).width(Length::Shrink)))
                        .width(Length::Fill)
                        .height(Length::Fill)
                } else {
                    scrollable(
                        container(text("Rendering page..."))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .center_y(Length::Fill),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                };

                column![toolbar, page_view].into()
            } else {
                text("No document loaded").into()
            };

            column![tab_bar, main_content].spacing(10).padding(10).into()
        };

        content
    }
}
