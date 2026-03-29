use rand::{Rng, RngExt, SeedableRng};
use rand::rngs::SmallRng;
use crate::signal_processing::generator::generator::Generator;

pub struct WhiteNoiseGenerator {
    random: rand::prelude::SmallRng
}

impl WhiteNoiseGenerator {
    pub fn new() -> WhiteNoiseGenerator {
        return WhiteNoiseGenerator{
            random: SmallRng::from_seed([67; 32])
        }
    }
}

impl Generator for WhiteNoiseGenerator {
    fn generate(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.random.random_range(-1.0..1.0);
        }
    }
}