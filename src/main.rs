use iced::widget::{Container, button, center, column, mouse_area, row, scrollable, text};
use iced::window::Position::Centered;
use iced::window::Settings;
use iced::window::icon::from_rgba;
use iced::{Element, Length, Size, Task as Command, Theme, alignment};
use mobi::Mobi;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use image::EncodableLayout;
use rust_embed::RustEmbed;
use walkdir::WalkDir;

#[derive(Debug, Default, Clone)]
pub struct KindleBook {
    title: String,
    description: Option<String>,
    author: Option<String>,
    isbn: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct State {
    kindle: Option<KindleDevice>,
    selected_book: Option<KindleBook>,
}

#[derive(Debug)]
enum Kindler {
    TryingToConnect,
    Connected(State),
    LoadedBooks(State),
}

#[derive(Debug, Default, Clone)]
pub struct KindleDevice {
    books: Vec<KindleBook>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    WaitingForDevice,
    LoadedBooks(State),
    Connected(State),
    SelectedBook(KindleBook),
}

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

pub fn main() -> iced::Result {
    let icon = {
        let icon_bytes = Asset::get("icon.png").unwrap().data;
        let image = image::load_from_memory(icon_bytes.as_bytes()).unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let raw = image.into_raw();
        Some(from_rgba(raw, width, height).unwrap())
    };

    iced::application("Kindler", Kindler::update, Kindler::view)
        .window_size((500.0, 800.0))
        .theme(|_| Theme::Dark)
        .window(Settings {
            icon,
            decorations: true,
            size: Size {
                width: 1200.0,
                height: 800.0,
            },
            position: Centered,
            ..Default::default()
        })
        .run_with(|| Kindler::new())
}

fn kindle_connected() -> bool {
    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        mounts.contains("Kindle")
    } else {
        false
    }
}

fn list_kindle_books(path: &str) -> std::io::Result<(Vec<KindleBook>)> {
    let path = Path::new(path);
    let mut on_device_books = Vec::new();
    if !path.exists() {
        println!("Path {} does not exist.", path.display());
        return Ok(on_device_books);
    }

    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        let file_path = entry.path();
        if let Some(ext) = file_path.extension() {
            if ["mobi"].contains(&ext.to_str().unwrap_or("")) {
                println!("Loading book... {}", file_path.to_str().unwrap());
                let mobi = Mobi::from_path(file_path).unwrap();
                on_device_books.push(KindleBook {
                    title: mobi.title(),
                    description: mobi.description(),
                    author: mobi.author(),
                    isbn: mobi.isbn(),
                })
            }
        }
    }
    Ok(on_device_books)
}

impl Kindler {
    async fn try_load_books() -> State {
        println!("try_load_books");

        let username = whoami::username();
        let mount_point = format!("/media/{}/Kindle/documents", username); // Replace with detected mount
        match list_kindle_books(&mount_point) {
            Ok(kindle_books) => State {
                kindle: Some(KindleDevice {
                    books: kindle_books,
                }),
                selected_book: None,
            },
            Err(error) => {
                println!("Error: {}", error);
                State {
                    kindle: Some(KindleDevice { books: Vec::new() }),
                    selected_book: None,
                }
            }
        }
    }

    async fn try_connect() -> State {
        let mut was_connected = false;
        while !was_connected {
            let now_connected = kindle_connected();
            was_connected = now_connected;
            thread::sleep(Duration::from_secs(2));
        }
        State {
            kindle: Some(KindleDevice { books: Vec::new() }),
            selected_book: None,
        }
    }

    fn new() -> (Self, Command<Message>) {
        (
            Self::TryingToConnect,
            Command::perform(Kindler::try_connect(), Message::Connected),
        )
    }

    pub fn view(&self) -> Element<Message> {
        match self {
            Kindler::TryingToConnect => {
                let content =
                    column![text("Please, connect your Kindle device").size(12)].spacing(20);
                center(content).padding(40).into()
            }
            Kindler::Connected(state) => {
                let content =
                    column![text("Connected to device. Reading books...").size(12)].spacing(20);
                center(content).padding(40).into()
            }
            Kindler::LoadedBooks(state) => {
                let kindle = state.clone().kindle.unwrap();
                let refresh_button = button(text("Refresh").size(14)).on_press(Message::Refresh).padding(8);
                let top_bar = row![
                    iced::widget::Space::with_width(Length::Fill),
                    refresh_button
                ]
                .padding(10);
                let mut books_rows = column![];
                for book in kindle.books {
                    let file_title = text(book.clone().title).size(15);
                    let file_author = text(book.clone().author.unwrap_or("".to_string())).size(14);
                    let view_button = button(text("View").size(14))
                        .on_press(Message::SelectedBook(book.clone()))
                        .padding(8);
                    let tags = text(match book.clone().description {
                        Some(_) => "Has info",
                        None => "",
                    })
                    .size(10);
                    let content = row![
                        column![view_button, tags].spacing(5),
                        column![file_title, file_author].spacing(5)
                    ]
                    .spacing(20)
                    .padding(20);
                    books_rows = books_rows.push(content);
                }
                let detailed_book_view = match &state.selected_book {
                    Some(book) => column![
                        text(
                            book.clone()
                                .description
                                .unwrap_or("No description provided".to_string())
                        )
                        .size(20)
                        .width(Length::Fill),
                        text(book.clone().isbn.unwrap_or("No ISBN provided".to_string()))
                            .size(20)
                            .width(Length::Fill)
                    ],
                    _ => column![text("").size(20).width(Length::Fill)],
                };
                let content = detailed_book_view
                    .spacing(20)
                    .padding(20)
                    .width(Length::Fill);
                let content = mouse_area(column![
                    top_bar,
                    row![
                        scrollable(books_rows).width(300).height(Length::Fill),
                        content
                    ],
                ]);
                Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(alignment::Horizontal::Left)
                    .align_y(alignment::Vertical::Top)
                    .into()
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Kindler::TryingToConnect => match message {
                Message::Connected(state) => {
                    *self = Kindler::Connected(state);
                    Command::perform(Kindler::try_load_books(), Message::LoadedBooks)
                }
                _ => Command::none(),
            },
            Kindler::Connected(state) => match message {
                Message::LoadedBooks(state) => {
                    *self = Kindler::LoadedBooks(state);
                    Command::none()
                }
                _ => Command::none(),
            },
            Kindler::LoadedBooks(state) => match message {
                Message::Connected(state) => {
                    *self = Kindler::Connected(state);
                    Command::none()
                }
                Message::Refresh => {
                    *self = Kindler::TryingToConnect;
                    Command::perform(Kindler::try_connect(), Message::Connected)
                }
                Message::SelectedBook(book) => {
                    *self = Kindler::LoadedBooks(State {
                        selected_book: Some(book),
                        ..state.clone()
                    });
                    Command::none()
                }
                _ => Command::none(),
            },
        }
    }
}
