use std::hash::{Hash, Hasher};
use std::sync::Arc;
use jack::{AudioIn, AudioOut};

pub trait AudioSink {
    fn init(&mut self, number_of_channels: usize);
}

pub enum SinkType {
    OneToOne,
    AllToOne
}

pub struct PortInfo {
    pub number: usize,
    pub input_port_name: String,
    pub output_port_name: String,
    pub channel: Channel,
}

pub struct Channel {
    pub input_port: jack::Port<AudioIn>,
    pub output_port: jack::Port<AudioOut>,
}

impl PortInfo {
    pub const INPUT_PORT_NAME_BASE: &'static str = "input_port_";
    pub const OUTPUT_PORT_NAME_BASE: &'static str = "output_port_";
}

impl Hash for PortInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.number.hash(state);
    }
}

impl PartialEq for PortInfo {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}