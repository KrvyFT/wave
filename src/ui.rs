use cpal::traits::StreamTrait;
use iced::widget::{column, container, slider, text};
use iced::{Center, Element, Fill};

use crate::audio::Wave;
use crate::effects::filter::Filter;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    EnableFilter,
    FilterSliderChanged(i32),
}

impl Wave {
    pub fn new() -> Self {
        let (input_stream, output_stream) = Self::create_stream();
        input_stream.play().unwrap();
        output_stream.play().unwrap();
        Self {
            input_stream,
            output_stream,
            filter: Filter::default(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::EnableFilter => {
                self.filter.used = true;
            }
            Message::FilterSliderChanged(alpha) => self.filter.alpha = alpha as f32,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let slider = container(
            slider(
                1..=1000,
                self.filter.alpha as i32,
                Message::FilterSliderChanged,
            )
            .default(500)
            .shift_step(5),
        )
        .width(250);

        let text = text(self.filter.alpha / 1000.0);

        column![slider, text,]
            .width(Fill)
            .align_x(Center)
            .spacing(20)
            .padding(20)
            .into()
    }
}

impl Default for Wave {
    fn default() -> Self {
        Self::new()
    }
}
