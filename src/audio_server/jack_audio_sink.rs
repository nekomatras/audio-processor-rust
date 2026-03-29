use super::audio_sink::{AudioSink, Channel, PortInfo, SinkType};
use crate::signal_processing::effect::lpf::{BaseLowPassFilter, ButterworthFilter2};
use crate::signal_processing::effect::effect::Effect;
use crate::utils::critical_error_handler;

use std::borrow::BorrowMut;
use std::{cell::RefCell, error::Error, str::FromStr};
use std::collections::HashMap;
use jack::AsyncClient;
use jack::{AudioIn, AudioOut, contrib::ClosureProcessHandler, PortFlags};

pub struct JackAudioSink {
    sink_type: SinkType,
    port_infos: HashMap<usize, PortInfo>,
    active_client: Option<jack::AsyncClient<(), ClosureProcessHandler<(), Box<dyn FnMut(&jack::Client, &jack::ProcessScope) -> jack::Control + Send>>>>
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
            port_infos: HashMap::new(),
            active_client: Some(active_client)
        })
    }

    pub fn register_handler(&mut self) {
        let (client, _, _) = self.active_client.take().unwrap().deactivate().unwrap();

        let mut channels = Vec::with_capacity(self.port_infos.len());
        for (_, port_info) in self.port_infos.iter_mut() {
            if let Some(channel) = port_info.channel.take() {
                channels.push(channel);
            }
        }

        const buf_size: usize = 1024;
        const quant: usize = 128;

        let sample_rate = client.sample_rate();
        println!("Sample rate: {}", sample_rate);
        let mut aboba = ButterworthFilter2::new(sample_rate as u32, 1000, buf_size);

        let mut buffer_in: [f32; buf_size] = [0.0; buf_size];
        let mut buffer_out: [f32; buf_size] = [0.0; buf_size];
        let mut index = 0;

        let callback: Box<dyn FnMut(&jack::Client, &jack::ProcessScope) -> jack::Control + Send> = Box::new(move |client: &jack::Client, ps: &jack::ProcessScope|
            -> jack::Control {
                for channel in channels.iter_mut() {
                    let input_buffer = channel.input_port.as_slice(ps);
                    let output_buffer = channel.output_port.as_mut_slice(ps);

                    let start_in = index as usize * quant;
                    (&mut buffer_in[start_in..start_in + quant]).copy_from_slice(input_buffer);

                    index = index + 1;

                    if index == (buf_size / quant) {
                        index = 0;
                        aboba.operate(buffer_in.as_slice(), buffer_out.as_mut_slice());
                    }

                    let start_out = index as usize * quant;
                    output_buffer.copy_from_slice(&buffer_out[start_out..start_out + quant]);
                }
                return jack::Control::Continue;
            });

        let process_handler = jack::contrib::ClosureProcessHandler::new(callback);
        self.active_client = Some(client.activate_async((), process_handler).unwrap());
    }

    fn register_one_to_one_ports(&mut self, number_of_channels: usize) {
        self.port_infos.reserve(number_of_channels);

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
    
                self.port_infos.insert(n, PortInfo{
                    number: n, 
                    input_port_name, 
                    output_port_name, 
                    channel: Some(Channel{input_port, output_port})
                });
            }
        }
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
            self.port_infos.clear();
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