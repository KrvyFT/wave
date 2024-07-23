use wave::audio::Wave;

fn main() {
    iced::run("Slider - Iced", Wave::update, Wave::view).unwrap()
}
