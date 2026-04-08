use crate::signal_processing::effect::effect::Effect;

pub struct FrameAccumulator {
    original_frame_size: usize,
    target_frame_size: usize,
    frame_ratio: usize,

    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
    index: usize,

    callback: Option<Box<dyn Fn(&[f32], &mut [f32])>>,
}

impl FrameAccumulator {
    pub fn new(original_frame_size: usize, target_frame_size: usize) -> FrameAccumulator {
        let mut accumulator = FrameAccumulator {
            original_frame_size: original_frame_size,
            target_frame_size: target_frame_size,
            frame_ratio: 0,
            input_buffer: Vec::new(),
            output_buffer: Vec::new(),
            index: 0,
            callback: None
        };

        if target_frame_size % original_frame_size != 0 {
            println!("Warning wrong frame sizes: Currnet: {}; New: {}; New - Current ratio (must be integer): {}",
                original_frame_size,
                target_frame_size,
                target_frame_size as f32 / original_frame_size as f32);
        }

        accumulator.frame_ratio = target_frame_size / original_frame_size;
        accumulator.input_buffer.resize(target_frame_size, 0.0);
        accumulator.output_buffer.resize(target_frame_size, 0.0);

        return accumulator;
    }

    pub fn register_callback(&mut self, callback: Box<dyn Fn(&[f32], &mut [f32])>) {
        self.callback = Some(callback);
    }

    pub fn clear_callback(&mut self) {
        self.callback = None;
    }

    fn get_input_slice(&mut self) -> &mut [f32] {
        let start = self.index * self.original_frame_size;
        return &mut self.input_buffer[start..start + self.original_frame_size];
    }

    fn get_output_slice(&self) -> &[f32] {
        let start = self.index * self.original_frame_size;
        return &self.output_buffer[start..start + self.original_frame_size];
    }
}

impl Effect for FrameAccumulator {
    fn operate(&mut self, input: &[f32], output: &mut [f32]) {
        if input.len() != self.original_frame_size || output.len() != self.original_frame_size {
            println!("Warning wrong buffer length: Quant: {}; Input: {}; Output: {}", self.original_frame_size, input.len(), output.len());
        }

        self.get_input_slice().copy_from_slice(input);

        self.index = self.index + 1;

        if self.index == self.frame_ratio {
            self.index = 0;
            if let Some(operate) = &self.callback {
                operate(self.input_buffer.as_slice(), self.output_buffer.as_mut_slice());
            }
        }

        output.copy_from_slice(self.get_output_slice());
    }

    fn reset(&mut self) {
        todo!()
    }

    fn get_info(&self) -> String {
        return format!("FrameAccumulator: {} -> {}", self.original_frame_size, self.target_frame_size);
    }
}