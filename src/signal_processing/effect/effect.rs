pub trait Effect {
    fn operate(&mut self, input: &[f32], output: &mut [f32]);
}