use crate::signal_processing::generator::generator::Generator;

pub struct HarmonicGenerator {
    sample_rate: f32,
    freq: f32,
    phase: f32
}

impl HarmonicGenerator {
    pub fn new(freq: f32, sample_rate: usize) -> HarmonicGenerator {
        return HarmonicGenerator{
            sample_rate: sample_rate as f32,
            freq: freq,
            phase: 0.0
        }
    }
}

impl Generator for HarmonicGenerator {
    fn generate(&mut self, output: &mut [f32]) {

        for sample in output.iter_mut() {
            *sample = (2.0 * std::f32::consts::PI * self.phase).sin();

            // Обновляем фазу
            self.phase += self.freq / self.sample_rate;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }
}