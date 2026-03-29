use std::process;
use crate::signal_processing::effect::effect::Effect;

use rustfft::{num_complex::ComplexFloat};



pub struct ButterworthFilter2 {
    signal_freq: u32,
    cut_freq: u32,
    b_coefficients: [f32; 3],
    a_coefficients: [f32; 2],

    buffer: Vec<f32>,

    y1: f32,
    y2: f32,
    x1: f32,
    x2: f32
}

impl ButterworthFilter2 {

    pub fn new(signal_freq: u32, cut_freq: u32, buffer_size: usize) -> ButterworthFilter2 {

        let k = ((std::f32::consts::PI * cut_freq as f32) / signal_freq as f32).tan();

        let mut filter = ButterworthFilter2 {
            signal_freq: signal_freq,
            cut_freq: cut_freq,
            b_coefficients: [0.0, 0.0, 0.0],
            a_coefficients: [0.0, 0.0],

            buffer: Vec::new(),

            y1: 0.0,
            y2: 0.0,
            x1: 0.0,
            x2: 0.0,
        };

        filter.buffer.resize(buffer_size, 0.0);

        let div = 1.0 + 2.0.sqrt() * k + k * k;
        let b02 = (k * k) / div;
        let b1 = b02 * 2.0;
        filter.b_coefficients[0] = b02;
        filter.b_coefficients[1] = b1;
        filter.b_coefficients[2] = b02;

        let a1 = (2.0 * (k * k - 1.0)) / div;
        let a2 = (1.0 - 2.0.sqrt() * k + k * k) / div;
        filter.a_coefficients[0] = a1;
        filter.a_coefficients[1] = a2;

        return filter;
    }


}

impl Effect for ButterworthFilter2 {
    fn operate(&mut self, input: &[f32], output: &mut [f32]) {
        for index in 0..input.len() {
            let mut value = self.b_coefficients[0] * input[index]
                + self.b_coefficients[1] * self.x1
                + self.b_coefficients[2] * self.x2
                - self.a_coefficients[0] * self.y1
                - self.a_coefficients[1] * self.y2;

            value = if value.abs() < 1e-20 { 0.0 } else { value };

            self.y2 = self.y1;
            self.y1 = value;
            self.x2 = self.x1;
            self.x1 = input[index];

            output[index] = value;
        }
    }
}







pub struct BaseLowPassFilter {
    internal_buffer: Vec<f32>,
    scale: usize,
    buffer_size: usize,
    read_index: usize,
    write_index: usize,
}

impl BaseLowPassFilter {

    pub fn new(buffer_size: usize, scale: usize) -> BaseLowPassFilter {
        let mut lpf = BaseLowPassFilter {
            internal_buffer: Vec::new(),
            scale: scale,
            buffer_size: buffer_size,
            read_index: 0,
            write_index: 0,
        };

        lpf.internal_buffer.resize(buffer_size * scale, 0.0);

        return lpf;
    }
}

impl Effect for BaseLowPassFilter {
    fn operate(&mut self, input: &[f32], output: &mut [f32]) {
        println!("Aboba: {}", input.len());
    }
}



