use wave::audio::Wave;

fn main() {
    iced::run("Wave", Wave::update, Wave::view).unwrap()
}
