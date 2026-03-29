use rustfft::{Fft, FftPlanner, FftDirection, num_complex::Complex, algorithm::Radix4};
use std::io;

mod utils;
mod signal_processing;


mod audio_server;
use audio_server::{audio_sink::{AudioSink, SinkType}, jack_audio_sink::{self, JackAudioSink}};
use signal_processing::generator::{generator::Generator, harmonic::HarmonicGenerator, white_noise::WhiteNoiseGenerator};

use utils::critical_error_handler;

fn main() {
    let gen_client = create_generators();
    let name = "aboba";
    let mut sink = JackAudioSink::new(name, SinkType::OneToOne).unwrap();
    sink.init(1);
    sink.register_handler();

    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    gen_client.deactivate().unwrap();
}










use jack::*;

fn create_generators() -> jack::AsyncClient<(), jack::contrib::ClosureProcessHandler<(), Box<dyn FnMut(&jack::Client, &jack::ProcessScope) -> jack::Control + Send + 'static>>> {
    // Создаем клиент JACK
    let (client, _status) =
        Client::new("rust_audio_gen", ClientOptions::NO_START_SERVER)
            .expect("Failed to create client");

    // Регистрируем выходной порт
    let mut out_sin = client
        .register_port("sinus", AudioOut::default())
        .unwrap();

    let mut out_whn = client
        .register_port("wnoise", AudioOut::default())
        .unwrap();

    let mut wn = WhiteNoiseGenerator::new();
    let mut sin = HarmonicGenerator::new(10000.0, client.sample_rate());

    // Обработчик аудио
    let mut process = move |_: &Client, ps: &ProcessScope| -> Control {
        wn.generate(out_whn.as_mut_slice(ps));
        sin.generate(out_sin.as_mut_slice(ps));
        Control::Continue
    };

    // Box the closure to match the expected type
    let boxed_process: Box<dyn FnMut(&jack::Client, &jack::ProcessScope) -> jack::Control + Send + 'static> = Box::new(process);

    // Активируем клиент
    let active_client = client
        .activate_async((), ClosureProcessHandler::new(boxed_process))
        .unwrap();

    return active_client;
}