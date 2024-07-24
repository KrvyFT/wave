use cpal::traits::StreamTrait;
use cpal::{InputCallbackInfo, OutputCallbackInfo};
use iced::widget::{button, column, container, slider, text};
use iced::{Center, Element, Fill};

use crate::audio::{Wave, AUDIO_CHANNELS, EFF_CHANNELS};
use crate::effects::filter::Filter;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    EnableFilter,
    FilterSliderChanged(i32),
}

impl Wave {
    pub fn new() -> Self {
        let host = Self::host();
        let (config, input_device, output_device) = Self::device_config(&host);
        let (input_stream, output_stream) =
            Self::create_stream(&config, &input_device, &output_device);
        input_stream.play().unwrap();
        output_stream.play().unwrap();
        Self {
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
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::EnableFilter => {
                self.filter.used = true;
                self.clear_stream();

                let (input_stream, output_stream) = self.change_stream(
                    move |data: &[f32], _info: &InputCallbackInfo| {
                        for &sample in data {
                            AUDIO_CHANNELS.0.send(sample).unwrap();
                        }
                    },
                    move |data: &mut [f32], _info: &OutputCallbackInfo| {
                        static mut FILTER: Filter = Filter {
                            used: true,
                            alpha: 0.0,
                        };

                        let mut alpha = unsafe { FILTER.alpha };
                        if let Ok(v) = EFF_CHANNELS.1.try_recv() {
                            println!("{}", v);
                            unsafe {
                                FILTER.alpha = v;
                            }
                            alpha = v;
                        } else {
                        };

                        for sample in data {
                            *sample = Filter::low_pass_filter(
                                alpha,
                                AUDIO_CHANNELS.1.recv().unwrap_or(0.0),
                            )
                        }
                    },
                );

                input_stream.play().unwrap();
                output_stream.play().unwrap();
                self.input_stream = Some(input_stream);
                self.output_stream = Some(output_stream);
            }
            Message::FilterSliderChanged(alpha) => {
                self.filter.alpha = alpha as f32;
                EFF_CHANNELS.0.send(alpha as f32).unwrap();
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
