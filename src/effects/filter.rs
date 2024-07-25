use cpal::OutputCallbackInfo;

use crate::audio::{AUDIO_CHANNELS, EFF_CHANNELS};

#[derive(Debug, Clone, Copy)]
pub struct Filter {
    pub used: bool,
    pub alpha: f32,
}

impl Filter {
    pub fn new() -> Self {
        Filter {
            used: false,
            alpha: 0.0,
        }
    }

    pub fn low_pass_filter(alpha: f32, sample: f32) -> f32 {
        let alpha = alpha / 1000.0;
        static mut LAST_SAMPLE: f32 = 0.0;
        unsafe {
            LAST_SAMPLE = alpha * sample + (1.0 - alpha) * LAST_SAMPLE;
            LAST_SAMPLE
        }
    }

    pub fn filter_handle_output() -> impl FnMut(&mut [f32], &OutputCallbackInfo) + Send + 'static {
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
                *sample = Filter::low_pass_filter(alpha, AUDIO_CHANNELS.1.recv().unwrap_or(0.0))
            }
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}
