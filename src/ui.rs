use std::sync::RwLock;

use iced::widget::{column, container, slider, text, vertical_slider};
use iced::{Center, Element, Fill};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FILTER: RwLock<i32> = {
        let v = 0;
        RwLock::new(v)
    };
}

#[derive(Debug)]
pub struct Filter {
    alpha: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    SliderChanged(i32),
}

impl Filter {
    pub fn new() -> Self {
        Self { alpha: 0 }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SliderChanged(v) => {
                *FILTER.write().unwrap() = v;
                self.alpha = v;
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let slider = container(
            slider(1..=1000, self.alpha, Message::SliderChanged)
                .default(50)
                .shift_step(5),
        )
        .width(250);

        let text = text(self.alpha as f32 / 1000.0);

        column![slider, text,]
            .width(Fill)
            .align_x(Center)
            .spacing(20)
            .padding(20)
            .into()
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}
