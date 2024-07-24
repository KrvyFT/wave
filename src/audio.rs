use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, InputCallbackInfo, OutputCallbackInfo, Stream, StreamConfig};
use ringbuf::storage::Heap;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::wrap::caching::Caching;
use ringbuf::{HeapRb, SharedRb};

use crate::effects::filter::Filter;

pub type producer = Caching<Arc<SharedRb<Heap<f32>>>, true, false>;
pub type consumer = Caching<Arc<SharedRb<Heap<f32>>>, false, true>;

pub struct Wave {
    pub host: Host,
    pub config: StreamConfig,
    pub input_device: Device,
    pub output_device: Device,
    pub input_stream: Option<Stream>,
    pub output_stream: Option<Stream>,
    pub filter: Filter,
}

pub struct SharedWave {
    pub wave: Arc<Mutex<Wave>>,
}

impl SharedWave {
    pub fn create_stream(
        config: &StreamConfig,
        input_device: &Device,
        output_device: &Device,
    ) -> (Stream, Stream) {
        let ring = HeapRb::<f32>::new(config.sample_rate.0 as usize * config.channels as usize);
        let (producer, consumer) = ring.split();

        let input_stream = input_device
            .build_input_stream(
                &config,
                Self::default_handle_input(producer),
                Self::err_fn,
                None,
            )
            .unwrap();
        let output_stream = output_device
            .build_output_stream(
                &config,
                Self::default_handle_output(consumer),
                Self::err_fn,
                None,
            )
            .unwrap();
        (input_stream, output_stream)
    }

    pub fn change_stream(
        self,
        handle_input: impl FnMut(&[f32], &InputCallbackInfo) + Send + 'static,
        handle_output: impl FnMut(&mut [f32], &OutputCallbackInfo) + Send + 'static,
    ) -> (Stream, Stream) {
        let guard = self.wave.lock().unwrap();
        let input_stream = guard
            .input_device
            .build_input_stream(&guard.config, handle_input, Self::err_fn, None)
            .unwrap();
        let output_stream = guard
            .output_device
            .build_output_stream(&guard.config, handle_output, Self::err_fn, None)
            .unwrap();
        (input_stream, output_stream)
    }

    pub fn create_buffer(&self) -> (producer, consumer) {
        let guard = self.wave.lock().unwrap();

        let ring = HeapRb::<f32>::new(
            guard.config.sample_rate.0 as usize * guard.config.channels as usize,
        );
        let (producer, consumer) = ring.split();
        (producer, consumer)
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

    pub fn default_handle_input(
        mut producer: ringbuf::wrap::caching::Caching<
            std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>,
            true,
            false,
        >,
    ) -> impl FnMut(&[f32], &InputCallbackInfo) + Send + 'static {
        move |data: &[f32], _info: &InputCallbackInfo| {
            for &sample in data {
                producer.try_push(sample).unwrap();
            }
        }
    }

    pub fn default_handle_output(
        mut consumer: ringbuf::wrap::caching::Caching<
            std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>,
            false,
            true,
        >,
    ) -> impl FnMut(&mut [f32], &OutputCallbackInfo) + Send + 'static {
        move |data: &mut [f32], _info: &OutputCallbackInfo| {
            for sample in data {
                *sample = consumer.try_pop().unwrap_or(0.0);
            }
        }
    }

    pub fn err_fn(err: cpal::StreamError) {
        eprintln!("an error occurred on stream: {}", err);
    }
}

impl Wave {
    pub fn change_stream(
        self,
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

    pub fn create_buffer(&self) -> (producer, consumer) {
        let ring =
            HeapRb::<f32>::new(self.config.sample_rate.0 as usize * self.config.channels as usize);
        let (producer, consumer) = ring.split();
        (producer, consumer)
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

    pub fn default_handle_input(
        mut producer: ringbuf::wrap::caching::Caching<
            std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>,
            true,
            false,
        >,
    ) -> impl FnMut(&[f32], &InputCallbackInfo) + Send + 'static {
        move |data: &[f32], _info: &InputCallbackInfo| {
            for &sample in data {
                producer.try_push(sample).unwrap();
            }
        }
    }

    pub fn default_handle_output(
        mut consumer: ringbuf::wrap::caching::Caching<
            std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>,
            false,
            true,
        >,
    ) -> impl FnMut(&mut [f32], &OutputCallbackInfo) + Send + 'static {
        move |data: &mut [f32], _info: &OutputCallbackInfo| {
            for sample in data {
                *sample = consumer.try_pop().unwrap_or(0.0);
            }
        }
    }

    pub fn err_fn(err: cpal::StreamError) {
        eprintln!("an error occurred on stream: {}", err);
    }
}
