#[macro_use]
extern crate vst;
use std::convert::TryInto;
use std::sync::{Arc, RwLock};
use arr_macro::arr;
use midi_consts::channel_event::CONTROL_CHANGE;

use vst::buffer::AudioBuffer;
use vst::buffer::SendEventBuffer;
use vst::event::MidiEvent;
use vst::plugin::{CanDo, HostCallback, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;

plugin_main!(CCControl); // Important!

static NUM_CCS: i32 = 32;

#[derive(Default)]
struct CCControl {
    host: HostCallback,
    params: Arc<CCControlParameters>,
    send_buffer: SendEventBuffer,
}

impl CCControl {
    fn send_midi(&mut self) {
        let mut stack = self.params.update_stack.write().unwrap();
        let mut index = stack.pop();
        let mut new_cc_events: Vec<MidiEvent> = Vec::new();
        while index.is_some() {
            let i: usize = index.unwrap();

            let channel_number: u8 = (i % 16).try_into().unwrap();
            let cc_number: u8 = ((i - (i % 16)) / 16).try_into().unwrap();

            let val = self.params.midi_cc[i].get();
            let val = (val * 127.0).round();
            let val = val as u8;

            let midi_data: [u8; 3] = [CONTROL_CHANGE | channel_number, cc_number, val];
            let midi_event = MidiEvent {
                data: midi_data,
                live: true,
                detune: 0,
                delta_frames: 0,
                note_length: None,
                note_offset: None,
                note_off_velocity: 0,
            };

            new_cc_events.push(midi_event);
            index = stack.pop();
        }

        self.send_buffer.send_events(new_cc_events, &mut self.host);
    }
}
impl Plugin for CCControl {
    fn new(host: HostCallback) -> Self {
        let mut p = CCControl::default();
        p.host = host;
        p
    }

    fn start_process(&mut self) {
        let mut stack = self.params.update_stack.write().unwrap();
        for cc in 0..NUM_CCS {
            for channel in 0..16 {
                let i: usize = (16 * cc + channel).try_into().unwrap();
                stack.push(i)
            }
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "CCController".to_string(),
            vendor: "Myrisa".to_string(),
            unique_id: 7357001, // Used by hosts to differentiate between plugins.
            version: 1,
            inputs: 2,
            outputs: 2,
            parameters: NUM_CCS * 16,
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

struct CCControlParameters {
    midi_cc: [AtomicFloat; 512],
    update_stack: RwLock<Vec<usize>>,
}

impl Default for CCControlParameters {
    fn default() -> CCControlParameters {
        CCControlParameters {
            midi_cc: arr![AtomicFloat::new(0.5); 512],
            update_stack: RwLock::new(Vec::new()),
        }
    }
}

impl PluginParameters for CCControlParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        let i: usize = index.try_into().unwrap();
        self.midi_cc[i].get()
    }

    fn get_parameter_text(&self, index: i32) -> String {
        let i: usize = index.try_into().unwrap();
        let val = self.midi_cc[i].get();
        let val = (val * 127.0).round();
        let val = val as u8;
        format!("{}", val)
    }

    fn get_parameter_name(&self, index: i32) -> String {
        let i = index;
        let channel_number = (i % 16) + 1;
        let cc_number = (i - (i % 16)) / 16;
        format!("CC {} # {}", cc_number, channel_number)
    }

    fn set_parameter(&self, index: i32, val: f32) {
        let i: usize = index.try_into().unwrap();
        self.midi_cc[i].set(val);
        let mut stack = self.update_stack.write().unwrap();
        stack.push(i)
    }
}
