use super::effect::effect::Effect;
use crate::audio_server::audio_sink::PortInfo;

pub struct Processor {
    port: PortInfo,
    effects: Vec<Box<dyn Effect>>,
    tmp_buffer: Vec<f32>
}

impl Processor {

    pub fn new(port_info: PortInfo) -> Processor {
        return Processor { port: (port_info), effects: Vec::new(), tmp_buffer: Vec::new() };
    }

    pub fn process(&mut self, ps: &jack::ProcessScope) {
        if self.effects.is_empty() {
            self.port.channel.output_port.as_mut_slice(ps).copy_from_slice(self.port.channel.input_port.as_slice(ps));
            return;
        }

        let tmp_buffer_len = self.tmp_buffer.len();
        let tmp_ptr = self.tmp_buffer.as_ptr();

        let mut output_buffer = self.port.channel.output_port.as_mut_slice(ps);

        if tmp_buffer_len < output_buffer.len() {
            self.tmp_buffer.resize(output_buffer.len(), 0.0);
        }

        let mut input_buffer = self.tmp_buffer.as_mut_slice();

        input_buffer.copy_from_slice(self.port.channel.input_port.as_slice(ps));

        for effect in &mut self.effects {
            effect.operate(input_buffer, output_buffer);
            std::mem::swap(&mut input_buffer, &mut output_buffer);
        }

        if std::ptr::eq(output_buffer.as_ptr(), tmp_ptr) {
            input_buffer.copy_from_slice(output_buffer);
        }
    }
}

unsafe impl Send for Processor {}