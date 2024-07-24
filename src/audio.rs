use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, InputCallbackInfo, OutputCallbackInfo, Stream, StreamConfig};
use crossbeam_channel::{bounded, Receiver, Sender};
use lazy_static::lazy_static;

use crate::effects::filter::Filter;

lazy_static! {
    pub static ref EFF_CHANNELS: (Sender<f32>, Receiver<f32>) = {
        let (tx, rx) = bounded(1);
        (tx, rx)
    };
    pub static ref AUDIO_CHANNELS: (Sender<f32>, Receiver<f32>) = {
        let (tx, rx) = bounded(1);
        (tx, rx)
    };
}

pub struct Wave {
    pub host: Host,
    pub config: StreamConfig,
    pub input_device: Device,
    pub output_device: Device,
    pub input_stream: Option<Stream>,
    pub output_stream: Option<Stream>,
    pub filter: Filter,
}

impl Wave {
    pub fn create_stream(
        config: &StreamConfig,
        input_device: &Device,
        output_device: &Device,
    ) -> (Stream, Stream) {
        let input_stream = input_device
            .build_input_stream(
                &config,
                move |data: &[f32], _info: &InputCallbackInfo| {
                    for &sample in data {
                        AUDIO_CHANNELS.0.send(sample).unwrap();
                    }
                },
                Self::err_fn,
                None,
            )
            .unwrap();
        let output_stream = output_device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _info: &OutputCallbackInfo| {
                    for sample in data {
                        *sample = AUDIO_CHANNELS.1.recv().unwrap_or(0.0);
                    }
                },
                Self::err_fn,
                None,
            )
            .unwrap();
        (input_stream, output_stream)
    }

    pub fn change_stream(
        &self,
        handle_input: impl FnMut(&[f32], &InputCallbackInfo) + Send + 'static,
        handle_output: impl FnMut(&mut [f32], &OutputCallbackInfo) + Send + 'static,
    ) -> (Stream, Stream) {
        let input_stream = self
            .input_device
            .build_input_stream(&self.config, handle_input, Self::err_fn, None)
            .unwrap();
        let output_stream = self
            .output_device
            .build_output_stream(&self.config, handle_output, Self::err_fn, None)
            .unwrap();
        (input_stream, output_stream)
    }

    pub fn device_config(host: &cpal::Host) -> (cpal::StreamConfig, Device, Device) {
        let input_device = host.default_input_device().expect("没有找到输入设备");
        let output_device = host.default_output_device().expect("没有找到输出设备");
        let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();
        (config, input_device, output_device)
    }

    pub fn host() -> Host {
        cpal::host_from_id(cpal::available_hosts()
            .into_iter()
            .find(|id| *id == cpal::HostId::Jack)
            .expect(
            "make sure --features jack is specified. only works on OSes where jack is available",
            )).expect("jack host unavailable")
    }

    pub fn err_fn(err: cpal::StreamError) {
        eprintln!("an error occurred on stream: {}", err);
    }

    pub fn clear_stream(&mut self) {
        drop(self.input_stream.take());
        drop(self.output_stream.take());
    }
}
