mod ui;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use ui::{Filter, FILTER};

fn main() {
    let host =
    cpal::host_from_id(cpal::available_hosts()
    .into_iter()
    .find(|id| *id == cpal::HostId::Jack)
    .expect(
    "make sure --features jack is specified. only works on OSes where jack is available",
    )).expect("jack host unavailable");

    let output_device = host.default_output_device().expect("没有找到输出设备");
    let inpue_device = host.default_input_device().expect("没有找到输入设备");

    let config: cpal::StreamConfig = inpue_device.default_input_config().unwrap().into();

    let ring = HeapRb::<f32>::new(config.sample_rate.0 as usize * config.channels as usize);
    let (mut producer, mut consumer) = ring.split();

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        for &sample in data {
            producer.try_push(sample).unwrap();
        }
    };

    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for sample in data {
            *sample = low_pass_filter(consumer.try_pop().unwrap_or(0.0));
        }
    };

    println!(
        "Attempting to build both streams with f32 samples and '{:?}'.",
        config
    );

    let input_stream = inpue_device
        .build_input_stream(&config, input_data_fn, err_fn, None)
        .unwrap();
    let output_stream = output_device
        .build_output_stream(&config, output_data_fn, err_fn, None)
        .unwrap();

    input_stream.play().unwrap();
    output_stream.play().unwrap();

    println!(
        "{:?}",
        iced::run("Slider - Iced", Filter::update, Filter::view).unwrap()
    );
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn low_pass_filter(sample: f32) -> f32 {
    // 简单的低通滤波器实现
    // 这里只是一个示例，实际应用中可能需要更复杂的滤波器
    let alpha = *FILTER.read().unwrap() as f32 / 1000.0; // 滤波器系数
    static mut LAST_SAMPLE: f32 = 0.0;
    unsafe {
        LAST_SAMPLE = alpha * sample + (1.0 - alpha) * LAST_SAMPLE;
        LAST_SAMPLE
    }
}
