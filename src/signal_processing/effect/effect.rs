pub trait Effect {
    fn operate(&mut self, input: &[f32], output: &mut [f32]);
    //fn frame_size_changed(&mut self, new_frame_size: usize);
    fn reset(&mut self);
    fn get_info(&self) -> String;
}