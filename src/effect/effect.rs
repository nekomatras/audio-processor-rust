pub trait Effect {
    fn process_input(&mut self, input: &[f32]);
    fn process_output(&mut self, output: &mut [f32]);
    fn operate(&mut self);
}