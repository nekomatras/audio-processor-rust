use std::process;
use crate::effect::effect::Effect;

use rustfft::{num_complex::ComplexFloat};



pub struct ButterworthFilter2 {
    signal_freq: u32,
    cut_freq: u32,
    b_coefficients: [f32; 3],
    a_coefficients: [f32; 2],

    buffer_size: usize,
    write_index: u8,
    read_index: u8,
    buffer: Vec<f32>,
}

impl ButterworthFilter2 {

    pub fn new(signal_freq: u32, cut_freq: u32, buffer_size: usize) -> ButterworthFilter2 {

        let k = ((std::f32::consts::PI * cut_freq as f32) / signal_freq as f32).tan();

        let mut filter = ButterworthFilter2 {
            signal_freq: signal_freq,
            cut_freq: cut_freq,
            b_coefficients: [0.0, 0.0, 0.0],
            a_coefficients: [0.0, 0.0],

            buffer_size: buffer_size,
            read_index: 1,
            write_index: 1,
            buffer: Vec::new(),
        };

        filter.buffer.resize(buffer_size * 2, 0.0);

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

    fn process_input(&mut self, input: &[f32]) {
        if input.len() != self.buffer_size {
            println!("Input buffer size [{}] not equal to filter buffer size [{}]", input.len(), self.buffer_size);
            process::exit(1);
        }

        let start = self.write_index as usize * self.buffer_size;
        let dst_part = &mut self.buffer[start..start + self.buffer_size];
        dst_part.copy_from_slice(input);

        self.write_index = (self.write_index + 1) % 2;
    }

    fn process_output(&mut self, output: &mut [f32]) {
        if output.len() != self.buffer_size {
            println!("Output buffer size [{}] not equal to filter buffer size [{}]", output.len(), self.buffer_size);
            process::exit(1);
        }

        let start = self.read_index as usize * self.buffer_size;
        output.copy_from_slice(&self.buffer[start..start + self.buffer_size]);

        self.read_index = (self.read_index + 1) % 2;
    }

    fn operate(&mut self) {
        // y[n]=b0​*x[n]+b1*​x[n−1]+b2*​x[n−2]−a1*​y[n−1]−a2*​y[n−2]

        let len = self.buffer.len();
        let original = self.buffer.clone();

        for index in 0..len {
            let value = self.b_coefficients[0] * original[index]
                + self.b_coefficients[1] * original[(len + index - 1) % len]
                + self.b_coefficients[2] * original[(len + index - 2) % len]
                - self.a_coefficients[0] * self.buffer[(len + index - 1) % len]
                - self.a_coefficients[1] * self.buffer[(len + index - 2) % len];

            self.buffer[index] = value;
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
    fn process_input(&mut self, input: &[f32]) {
        if input.len() != self.buffer_size {
            println!("Input buffer size [{}] not equal to filter buffer size [{}]", input.len(), self.buffer_size);
            process::exit(1);
        }

        let start = self.write_index * self.buffer_size;
        let dst_part = &mut self.internal_buffer[start..start + self.buffer_size];
        dst_part.copy_from_slice(input);

        self.write_index = (self.write_index + 1) % self.scale;
    }

    fn process_output(&mut self, output: &mut [f32]) {
        if output.len() != self.buffer_size {
            println!("Output buffer size [{}] not equal to filter buffer size [{}]", output.len(), self.buffer_size);
            process::exit(1);
        }

        let start = self.read_index * self.buffer_size;
        output.copy_from_slice(&self.internal_buffer[start..start + self.buffer_size]);

        self.read_index = (self.read_index + 1) % self.scale;
    }

    fn operate(&mut self) {

        let original = self.internal_buffer.clone();

        for (index, value) in self.internal_buffer.iter_mut().enumerate() {
            *value = 0.0;
            for ind in 0..self.scale {
                let real_index = (index + original.len() - ind) % (self.buffer_size * self.scale);
                let operating_value = original[real_index];
                *value += (operating_value / self.scale as f32);
            }
        }
    }
}



