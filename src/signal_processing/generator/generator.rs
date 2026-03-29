pub trait Generator {
    fn generate(&mut self, output: &mut [f32]);
}