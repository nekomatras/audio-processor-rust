use super::audio_sink::{AudioSink, Channel, PortInfo, SinkType};
use crate::signal_processing::effect::bpf::{ButterworthFilter2};
use crate::signal_processing::effect::effect::Effect;
use crate::utils::critical_error_handler;
use crate::signal_processing::processor::Processor;

use std::sync::{Arc, Mutex};
use std::borrow::BorrowMut;
use std::{cell::RefCell, error::Error, str::FromStr};
use std::collections::HashMap;
use jack::AsyncClient;
use jack::{AudioIn, AudioOut, contrib::ClosureProcessHandler, PortFlags};

pub struct JackAudioSink {
    sink_type: SinkType,
    //port_infos: HashMap<usize, PortInfo>,
    pub active_client: Option<jack::AsyncClient<(), ClosureProcessHandler<(), Box<dyn FnMut(&jack::Client, &jack::ProcessScope) -> jack::Control + Send>>>>,

    processors: Arc<Mutex<Vec<Processor>>>,
}

impl JackAudioSink {
    pub fn new(client_name: &str, sink_type: SinkType) -> Result<Self, jack::Error> {

        let (client, _) = jack::Client::new(client_name, jack::ClientOptions::default())?;

        let callback: Box<dyn FnMut(&jack::Client, &jack::ProcessScope) -> jack::Control + Send> = Box::new(move |_: &jack::Client, _: &jack::ProcessScope|
            -> jack::Control {
                return jack::Control::Continue;
            });

        let process_handler = jack::contrib::ClosureProcessHandler::new(callback);
        let active_client = client.activate_async((), process_handler)?;

        return Ok(Self {
            sink_type,
            //port_infos: HashMap::new(),
            active_client: Some(active_client),
            processors: Arc::new(Mutex::new(Vec::new()))
        })
    }

    pub fn register_handler(&mut self) {
        let (client, _, _) = self.active_client.take().unwrap().deactivate().unwrap();

        let sample_rate = client.sample_rate();
        println!("Sample rate: {}", sample_rate);

        let proc_copy = self.processors.clone();

        let callback: Box<dyn FnMut(&jack::Client, &jack::ProcessScope) -> jack::Control + Send> = Box::new(move |client: &jack::Client, ps: &jack::ProcessScope|
            -> jack::Control {
                let mut processors = proc_copy.lock().expect("Can't lock processors ptr copy");
                for processor in processors.iter_mut() {
                    processor.process(ps);
                }
                return jack::Control::Continue;
            });

        let process_handler = jack::contrib::ClosureProcessHandler::new(callback);
        self.active_client = Some(client.activate_async((), process_handler).unwrap());
    }

    fn register_one_to_one_ports(&mut self, number_of_channels: usize) {
        let mut tmp_processors: Vec<Processor> = Vec::new();
        tmp_processors.reserve(number_of_channels);

        if let Some(active_client) = self.active_client.as_mut() {
            let client = active_client.as_client();

            for n in 1..=number_of_channels {
                let mut input_port_name = String::from_str(PortInfo::INPUT_PORT_NAME_BASE).unwrap();
                input_port_name.push_str(&n.to_string());
                let mut output_port_name = String::from_str(PortInfo::OUTPUT_PORT_NAME_BASE).unwrap();
                output_port_name.push_str(&n.to_string());

                let input_port = client.register_port(&input_port_name, jack::AudioIn::default())
                    .unwrap_or_else(|error| { critical_error_handler(&error.to_string()); });
                let output_port = client.register_port(&output_port_name, jack::AudioOut::default())
                    .unwrap_or_else(|error| { critical_error_handler(&error.to_string()); });

                let mut proc = Processor::new(PortInfo{
                    number: n,
                    input_port_name,
                    output_port_name,
                    channel: Channel{input_port, output_port}
                });

                let filter = ButterworthFilter2::new(48000, 5000, 10000, 128);
                proc.effects.push(Box::new(filter));

                tmp_processors.push(proc);
            }
        }

        let mut processors: std::sync::MutexGuard<'_, Vec<Processor>> = self.get_processors_mut();
        processors.clear();
        processors.reserve(number_of_channels);
        processors.append(&mut tmp_processors);
    }


    fn register_all_to_one_ports(&mut self, _: usize) {
        critical_error_handler("All to One mode not implemented yet!");
    }

    fn reset_ports_data(&mut self) {
        if let Some(active_client) = self.active_client.as_mut() {
            let client = active_client.as_client();
            let mut port_name = String::from_str(client.name()).unwrap();
            port_name.push_str(":.*");
            for name in client.ports(Some(&port_name), None, PortFlags::empty()).iter() {
                if let Some(port) = client.port_by_name(name) {
                    client.unregister_port(port)
                       .unwrap_or_else(|error| { critical_error_handler(&error.to_string()); });
                }
            }
            self.get_processors().clear();
        }
    }

    fn port_name_regex(&mut self) -> Result<String, &'static str>{
        if let Some(active_client) = self.active_client.as_mut() {
            let client = active_client.as_client();
            let mut port_name_regex = String::new();
            port_name_regex.push_str(client.name());
            port_name_regex.push_str(":.*");
            return Ok(port_name_regex);
        }
        return Err("Client is inactive!");
    }

    fn get_processors(&self) -> std::sync::MutexGuard<'_, Vec<Processor>> {
        return self.processors.lock().expect("Unable to lock mutex for processors");
    }

    fn get_processors_mut(&mut self) -> std::sync::MutexGuard<'_, Vec<Processor>> {
        return self.processors.lock().expect("Unable to lock mutex for processors");
    }
}

impl AudioSink for JackAudioSink {
    fn init(&mut self, number_of_channels: usize) {
        self.reset_ports_data();
        match self.sink_type {
            SinkType::OneToOne => {
                self.register_one_to_one_ports(number_of_channels);
            }
            SinkType::AllToOne => {
                self.register_all_to_one_ports(number_of_channels);
            }
        }
    }
}