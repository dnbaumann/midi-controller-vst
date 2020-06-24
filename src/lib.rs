#[macro_use]
extern crate vst;
use std::convert::TryInto;
use std::sync::Arc;

use arr_macro::arr;

use vst::buffer::SendEventBuffer;
use vst::util::AtomicFloat;
use vst::plugin::{CanDo, HostCallback, Info, Plugin, PluginParameters};

plugin_main!(MidiControl); // Important!

#[derive(Default)]
struct MidiControl {
    host: HostCallback,
    params: Arc<MidiControlParameters>,
    send_buffer: SendEventBuffer,
}

impl MidiControl {
    fn send_midi(&mut self) {
        while params.update_stack.len() != 0 {
            let index: usize = params.update_statck.pop();
            self.send_buffer.send_events(&self.events, &mut self.host);
        }
    }
}


impl Plugin for MidiControl {
    fn new(host: HostCallback) -> Self {
        let mut p = MidiControl::default();
        p.host = host;
        p
    }

    fn get_info(&self) -> Info {
        Info {
            name: "MidiControl".to_string(),
            vendor: "Michael Muszynski".to_string(),
            unique_id: 7357001, // Used by hosts to differentiate between plugins.
            version: 1,
            inputs: 2,
            outputs: 2,
            parameters: 2048,
            ..Default::default()
        }
    }

    fn can_do(&self, can_do: CanDo) -> vst::api::Supported {
        use vst::api::Supported::*;
        use vst::plugin::CanDo::*;

        match can_do {
            SendEvents | SendMidiEvent | ReceiveEvents | ReceiveMidiEvent => Yes,
            _ => No,
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        for (input, output) in buffer.zip() {
            for (in_sample, out_sample) in input.iter().zip(output) {
                *out_sample = *in_sample;
            }
        }
        self.send_midi();
    }

    fn process_f64(&mut self, buffer: &mut AudioBuffer<f64>) {
        for (input, output) in buffer.zip() {
            for (in_sample, out_sample) in input.iter().zip(output) {
                *out_sample = *in_sample;
            }
        }
        self.send_midi();
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

struct MidiControlParameters {
    midi_cc: [AtomicFloat; 2048 /*128*16*/],
    update_stack: Vec<usize>,
}

impl Default for MidiControlParameters {
    fn default() -> MidiControlParameters {
        MidiControlParameters {
            midi_cc: arr![AtomicFloat::new(0.0); 2048 /*128*16*/]
        }
    }
}

impl PluginParameters for MidiControlParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        let i: usize = index.try_into().unwrap();
        self.midi_cc[i].get()
    }

    fn get_parameter_text(&self, index: i32) -> String {
        let i: usize = index.try_into().unwrap();
        let value = self.midi_cc[i].get();
        format!("{}", value)
    }

    fn get_parameter_name(&self, index: i32) -> String {
        let midi_channel_number = (index % 16) + 1;
        let cc_number = (index - midi_channel_number - 1) / 16;
        format!("CC {} # {}", cc_number, midi_channel_number)
    }

    fn set_parameter(&self, index: i32, val: f32) {
        let i: usize = index.try_into().unwrap();
        self.midi_cc[i].set(val);
        self.update_stack.push(i)
    }
}
