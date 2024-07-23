use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, InputCallbackInfo, OutputCallbackInfo, Stream};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;

use crate::effects::filter::Filter;

pub struct Wave {
    pub input_stream: Stream,
    pub output_stream: Stream,
    pub filter: Filter,
}

impl Wave {
    pub fn create_stream() -> (Stream, Stream) {
        let host =
        cpal::host_from_id(cpal::available_hosts()
        .into_iter()
        .find(|id| *id == cpal::HostId::Jack)
        .expect(
        "make sure --features jack is specified. only works on OSes where jack is available",
        )).expect("jack host unavailable");

        let (config, input_device, output_device) = Self::device_config(host);

        let ring = HeapRb::<f32>::new(config.sample_rate.0 as usize * config.channels as usize);
        let (producer, consumer) = ring.split();

        let input_stream = input_device
            .build_input_stream(&config, Self::handle_input(producer), Self::err_fn, None)
            .unwrap();
        let output_stream = output_device
            .build_output_stream(&config, Self::handle_output(consumer), Self::err_fn, None)
            .unwrap();
        (input_stream, output_stream)
    }

    fn device_config(host: cpal::Host) -> (cpal::StreamConfig, Device, Device) {
        let input_device = host.default_input_device().expect("没有找到输入设备");
        let output_device = host.default_output_device().expect("没有找到输出设备");
        let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();
        (config, input_device, output_device)
    }

    fn handle_input(
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

    fn handle_output(
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

    fn err_fn(err: cpal::StreamError) {
        eprintln!("an error occurred on stream: {}", err);
    }
}
