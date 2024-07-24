use std::sync::{Arc, Mutex};

use cpal::traits::StreamTrait;
use cpal::{InputCallbackInfo, OutputCallbackInfo};
use iced::widget::{button, column, container, slider, text};
use iced::{Center, Element, Fill};
use ringbuf::traits::{Consumer, Producer};

use crate::audio::{SharedWave, Wave};
use crate::effects::filter::Filter;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    EnableFilter,
    FilterSliderChanged(i32),
}

impl SharedWave {
    pub fn new() -> Self {
        let host = Self::host();
        let (config, input_device, output_device) = Self::device_config(&host);
        let (input_stream, output_stream) =
            Self::create_stream(&config, &input_device, &output_device);
        input_stream.play().unwrap();
        output_stream.play().unwrap();
        Self {
            wave: Arc::new(Mutex::new(Wave {
                host,
                config,
                input_device,
                output_device,
                input_stream: Some(input_stream),
                output_stream: Some(output_stream),
                filter: Filter {
                    used: false,
                    alpha: 0.0,
                },
            })),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::EnableFilter => {
                let mut guard = *self.wave.lock().unwrap();
                guard.filter.used = true;
                drop(guard.input_stream.take());
                drop(guard.output_stream.take());

                let (mut producer, mut consumer) = self.create_buffer();
                let filter = guard.filter;

                let (input_stream, output_stream) = guard.change_stream(
                    move |data: &[f32], _info: &InputCallbackInfo| {
                        for &sample in data {
                            producer.try_push(sample).unwrap();
                        }
                    },
                    move |data: &mut [f32], _info: &OutputCallbackInfo| {
                        for sample in data {
                            *sample = filter.low_pass_filter(consumer.try_pop().unwrap_or(0.0));
                            println!("{}", filter.alpha);
                        }
                    },
                );

                input_stream.play().unwrap();
                output_stream.play().unwrap();

                guard.input_stream = Some(input_stream);
                guard.output_stream = Some(output_stream);
            }
            Message::FilterSliderChanged(alpha) => {
                let guard = self.wave.lock().unwrap();
                guard.filter.alpha = alpha as f32
            }
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
        let button = button("Filter").on_press(Message::EnableFilter);
        column![button, slider, text,]
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
