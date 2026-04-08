use crate::signal_processing::effect::effect::Effect;

pub struct ButterworthFilter2 {
    signal_freq: u32,
    cut_freq_low: u32,
    cut_freq_high: u32,

    cut_freq: f32,
    b_coefficients: [f32; 3],
    a_coefficients: [f32; 3],

    buffer: Vec<f32>,

    y1: f32,
    y2: f32,
    x1: f32,
    x2: f32
}

impl ButterworthFilter2 {
    pub fn new(signal_freq: u32, cut_freq_low: u32, cut_freq_high: u32, buffer_size: usize) -> ButterworthFilter2 {

        let f0 = (cut_freq_low + cut_freq_high) as f32 / 2.0;
        let q = f0 / (cut_freq_high - cut_freq_low) as f32;
        let w0 = 2.0 * std::f32::consts::PI * (f0 / signal_freq as f32);
        let alpha = w0.sin() / (2.0 * q);

        let mut filter = ButterworthFilter2 {
            signal_freq: signal_freq,
            cut_freq_low: cut_freq_low,
            cut_freq_high: cut_freq_high,
            cut_freq: f0,
            b_coefficients: [0.0, 0.0, 0.0],
            a_coefficients: [0.0, 0.0, 0.0],

            buffer: Vec::new(),

            y1: 0.0,
            y2: 0.0,
            x1: 0.0,
            x2: 0.0,
        };

        filter.buffer.resize(buffer_size, 0.0);

        let b0: f32 = alpha;
        let b1: f32 = 0.0;
        let b2: f32 = -alpha;
        let a0: f32 = 1.0 + alpha;
        let a1: f32 = -2.0 * w0.cos();
        let a2: f32 = 1.0 - alpha;

        filter.b_coefficients[0] = b0 / a0;
        filter.b_coefficients[1] = b1 / a0;
        filter.b_coefficients[2] = b2 / a0;
        filter.a_coefficients[0] = a0;
        filter.a_coefficients[1] = a1 / a0;
        filter.a_coefficients[2] = a2 / a0;

        return filter;
    }
}

impl Effect for ButterworthFilter2 {
    fn operate(&mut self, input: &[f32], output: &mut [f32]) {
        for index in 0..input.len() {
            let mut value = self.b_coefficients[0] * input[index]
                + self.b_coefficients[1] * self.x1
                + self.b_coefficients[2] * self.x2
                - self.a_coefficients[1] * self.y1
                - self.a_coefficients[2] * self.y2;

            value = if value.abs() < 1e-20 { 0.0 } else { value };

            self.y2 = self.y1;
            self.y1 = value;
            self.x2 = self.x1;
            self.x1 = input[index];

            output[index] = value;
        }
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    fn get_info(&self) -> String {
        return format!("Butterworth BPF 2: fc: {}", self.cut_freq);
    }
}