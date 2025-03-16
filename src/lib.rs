use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::AudioContext;
use rand::Rng;
use std::sync::{Arc, Mutex};
use js_sys::Date;

mod audio;

use crate::audio::synth::{KarplusStrong, Oscillator, WaveType};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct LoFiPlayer {
    audio_ctx: AudioContext,
    samples: Arc<Mutex<Vec<f32>>>,
    is_playing: Arc<Mutex<bool>>,
    next_start_time: Arc<Mutex<f64>>, // For scheduling buffers
    vis_samples: Arc<Mutex<Vec<f32>>>, // Separate buffer for visualization
}

#[wasm_bindgen]
impl LoFiPlayer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<LoFiPlayer, JsValue> {
        let audio_ctx = AudioContext::new()?;
        let samples = Arc::new(Mutex::new(Vec::new()));
        let is_playing = Arc::new(Mutex::new(false));
        let next_start_time = Arc::new(Mutex::new(0.0));
        let vis_samples = Arc::new(Mutex::new(Vec::new()));
        Ok(LoFiPlayer {
            audio_ctx,
            samples,
            is_playing,
            next_start_time,
            vis_samples,
        })
    }

    #[wasm_bindgen]
    pub fn start_audio(&self) -> Result<(), JsValue> {
        let mut is_playing = self.is_playing.lock().unwrap();
        if *is_playing {
            return Ok(());
        }
        *is_playing = true;

        // Resume audio context on start
        self.audio_ctx.resume().unwrap_or_else(|e| {
            log(&format!("Failed to resume audio context: {:?}", e));
        });

        let samples_clone = Arc::clone(&self.samples);
        let vis_samples_clone = Arc::clone(&self.vis_samples);
        let audio_ctx = self.audio_ctx.clone();
        let is_playing_clone = Arc::clone(&self.is_playing);
        let next_start_time_clone = Arc::clone(&self.next_start_time);

        // Initialize next_start_time with current time
        let mut next_time = next_start_time_clone.lock().unwrap();
        *next_time = audio_ctx.current_time();
        log(&format!("Initial next_start_time set to: {}", *next_time));

        spawn_local(async move {
            let sample_rate = 44100.0;
            let bpm = 80.0;
            let beat_duration = 60.0 / bpm;
            let delay_size = (sample_rate * 0.45) as usize;
            let buffer_size = 1024;
            let buffer_duration = buffer_size as f64 / sample_rate as f64;

            let voices = Arc::new(Mutex::new(Vec::new()));
            let melody = Arc::new(Mutex::new(Melody {
                osc: KarplusStrong::new(440.0, sample_rate),
                gain: 0.2,
                decay: 0.02,
                time: 0.0,
                vibrato: 0.0,
                delay_buffer: vec![0.0; delay_size],
                delay_pos: 0,
            }));
            let bass = Arc::new(Mutex::new(Oscillator::new(87.31, sample_rate, WaveType::Sine)));
            let kick = Arc::new(Mutex::new(Drum {
                osc: Oscillator::new(50.0, sample_rate, WaveType::Sine),
                gain: 0.8,
                decay: 0.03,
                time: 0.0,
                filter: 0.0,
            }));
            let hihat = Arc::new(Mutex::new(Drum {
                osc: Oscillator::new(300.0, sample_rate, WaveType::Sine),
                gain: 0.08,
                decay: 0.12,
                time: 0.0,
                filter: 0.0,
            }));
            let snare = Arc::new(Mutex::new(Drum {
                osc: Oscillator::new(120.0, sample_rate, WaveType::Sine),
                gain: 0.25,
                decay: 0.06,
                time: 0.0,
                filter: 0.0,
            }));

            let voices_clone = Arc::clone(&voices);
            let melody_clone = Arc::clone(&melody);
            let bass_clone = Arc::clone(&bass);
            let kick_clone = Arc::clone(&kick);
            let hihat_clone = Arc::clone(&hihat);
            let snare_clone = Arc::clone(&snare);

            let mut time = 0.0;
            let sample_duration = 1.0 / sample_rate;
            let chords = vec![
                vec![220.00, 261.63, 329.63, 392.00, 523.25],
                vec![174.61, 261.63, 349.23, 440.00, 587.33],
                vec![130.81, 261.63, 329.63, 392.00, 659.25],
                vec![196.00, 246.94, 329.63, 392.00, 587.33],
            ];
            let melody_notes = vec![
                440.0, 523.25, 587.33, 659.25, 783.99, 880.0,
                987.77, 783.99, 659.25, 587.33, 523.25, 392.00,
            ];
            let mut chord_index = 0;
            let mut last_chord_time = -beat_duration;
            let mut melody_index = 0;
            let mut last_melody_time = 0.0;
            let mut swing_offset = 0.0;
            let mut output_filter = 0.0;

            let mut iteration = 0;
            let max_iterations = 1000;

            while *is_playing_clone.lock().unwrap() {
                let mut sample = 0.0;

                if time - last_chord_time >= beat_duration * 2.0 {
                    let mut voices = voices_clone.lock().unwrap();
                    voices.clear();
                    let chord = &chords[chord_index];
                    for (i, &freq) in chord.iter().enumerate() {
                        let detune = rand::thread_rng().gen_range(-3.0..3.0);
                        let offset = i as f32 * beat_duration * 0.12 + rand::thread_rng().gen_range(-0.06..0.06) * beat_duration;
                        let rand_gain = rand::thread_rng().gen_range(0.8..1.2);
                        voices.push(Voice {
                            osc: KarplusStrong::new(freq + detune, sample_rate),
                            sine: Oscillator::new(freq * 0.99, sample_rate, WaveType::Sine),
                            gain: 0.45 * rand_gain,
                            decay: 0.015,
                            time: offset,
                            delay_buffer: vec![0.0; delay_size],
                            delay_pos: 0,
                            offset,
                            drift: rand::thread_rng().gen_range(-0.002..0.002),
                        });
                    }
                    chord_index = (chord_index + 1) % chords.len();
                    last_chord_time = time;
                }

                if time - last_melody_time >= beat_duration * 0.5 {
                    let mut melody = melody_clone.lock().unwrap();
                    melody.osc = KarplusStrong::new(melody_notes[melody_index], sample_rate);
                    melody.time = rand::thread_rng().gen_range(-0.05..0.05) * beat_duration;
                    melody.vibrato = 0.0;
                    melody_index = (melody_index + 1) % melody_notes.len();
                    last_melody_time = time;
                }

                let mut voices = voices_clone.lock().unwrap();
                for voice in voices.iter_mut() {
                    let env = 1.0 - ((voice.time - voice.offset) * voice.decay).min(1.0);
                    voice.drift += rand::thread_rng().gen_range(-0.0001..0.0001);
                    let s = voice.osc.next_sample(voice.drift) * voice.gain * env * 0.1 +
                            voice.sine.next_sample() * voice.gain * 0.9 * env;
                    let delay_pos = voice.delay_pos;
                    let delayed = voice.delay_buffer[delay_pos] * 0.9;
                    sample += s + delayed;
                    voice.delay_buffer[delay_pos] = s * 0.5;
                    voice.delay_pos = (voice.delay_pos + 1) % voice.delay_buffer.len();
                    voice.time += sample_duration;
                }

                let mut melody = melody_clone.lock().unwrap();
                let env = 1.0 - (melody.time * melody.decay).min(1.0);
                melody.vibrato += 0.06;
                let vibrato = ((melody.vibrato * 0.3) as f32).sin() * 0.008;
                let s = melody.osc.next_sample(vibrato) * melody.gain * env;
                let delay_pos = melody.delay_pos;
                let delayed = melody.delay_buffer[delay_pos] * 0.8;
                sample += s + delayed;
                melody.delay_buffer[delay_pos] = s * 0.45;
                melody.delay_pos = (delay_pos + 1) % melody.delay_buffer.len();
                melody.time += sample_duration;

                let mut bass = bass_clone.lock().unwrap();
                let bass_env = ((time * 0.004) as f32).sin().abs() * 0.7 + 0.3;
                sample += bass.next_sample() * 0.7;

                let beat_time = time % (beat_duration * 4.0);
                let mut kick = kick_clone.lock().unwrap();
                let mut hihat = hihat_clone.lock().unwrap();
                let mut snare = snare_clone.lock().unwrap();

                if beat_time < beat_duration * 0.3 {
                    let env = 1.0 - (kick.time * kick.decay).min(1.0);
                    let rand_gain = rand::thread_rng().gen_range(0.9..1.1);
                    let s = kick.osc.next_sample() * kick.gain * env * rand_gain;
                    kick.filter += (s - kick.filter) * 0.04;
                    sample += kick.filter;
                    kick.time += sample_duration;
                } else {
                    kick.time = 0.0;
                }

                let hihat_time = beat_time % (beat_duration * 0.5);
                if hihat_time < beat_duration * 0.12 + swing_offset {
                    let env = 1.0 - (hihat.time * hihat.decay).min(1.0);
                    let rand_amp = rand::thread_rng().gen_range(0.7..1.3);
                    let s = hihat.osc.next_sample() * hihat.gain * env * rand_amp;
                    hihat.filter += (s - hihat.filter) * 0.06;
                    sample += hihat.filter;
                    hihat.time += sample_duration;
                    swing_offset = rand::thread_rng().gen_range(-0.05..0.05) * beat_duration;
                } else {
                    hihat.time = 0.0;
                }

                if beat_time >= beat_duration * 2.0 && beat_time < beat_duration * 2.3 {
                    let env = 1.0 - (snare.time * snare.decay).min(1.0);
                    let rand_gain = rand::thread_rng().gen_range(0.8..1.2);
                    let s = snare.osc.next_sample() * snare.gain * env * rand_gain;
                    snare.filter += (s - snare.filter) * 0.05;
                    sample += snare.filter;
                    snare.time += sample_duration;
                } else {
                    snare.time = 0.0;
                }

                output_filter += (sample - output_filter) * 0.04;
                let tremolo = ((time * 0.2) as f32).sin() * 0.06 + 0.94;
                sample = output_filter * tremolo;
                sample = sample.clamp(-0.8, 0.8);

                let mut samples = samples_clone.lock().unwrap();
                samples.push(sample);
                let mut vis_samples = vis_samples_clone.lock().unwrap();
                vis_samples.push(sample); // Duplicate for visualization

                if samples.len() >= buffer_size {
                    let buffer = match audio_ctx.create_buffer(1, buffer_size as u32, sample_rate) {
                        Ok(b) => b,
                        Err(e) => {
                            log(&format!("Failed to create buffer: {:?}", e));
                            return;
                        }
                    };
                    match buffer.get_channel_data(0) {
                        Ok(mut channel_data) => {
                            for (i, &s) in samples.iter().enumerate() {
                                channel_data[i] = s;
                            }
                        }
                        Err(e) => {
                            log(&format!("Failed to get channel data: {:?}", e));
                            return;
                        }
                    }
                    let source = match audio_ctx.create_buffer_source() {
                        Ok(s) => s,
                        Err(e) => {
                            log(&format!("Failed to create source: {:?}", e));
                            return;
                        }
                    };
                    source.set_buffer(Some(&buffer));
                    if let Err(e) = source.connect_with_audio_node(&audio_ctx.destination()) {
                        log(&format!("Failed to connect source: {:?}", e));
                        return;
                    }

                    let mut next_time = next_start_time_clone.lock().unwrap();
                    let current_time = audio_ctx.current_time();
                    log(&format!("Current audio context time: {}", current_time));
                    let start_time = next_time.max(current_time);
                    if let Err(e) = source.start_with_when(start_time) {
                        log(&format!("Failed to start source: {:?}", e));
                        return;
                    }
                    *next_time = start_time + buffer_duration;
                    log(&format!("Playing buffer at time: {}", start_time));

                    samples.clear();
                }

                time += sample_duration;
                iteration += 1;

                // Throttle the loop
                if iteration >= max_iterations {
                    log(&format!("Yielding after {} iterations", iteration));
                    iteration = 0;
                    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::NULL)).await.unwrap();
                }
            }
        });
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_sample(&self) -> Option<f64> {
        let vis_samples = self.vis_samples.lock().unwrap();
        if vis_samples.is_empty() {
            log("Visualization sample buffer empty");
            None
        } else {
            let index = vis_samples.len() - 1;
            Some(vis_samples[index] as f64)
        }
    }
}

#[derive(Clone)]
struct Voice {
    osc: KarplusStrong,
    sine: Oscillator,
    gain: f32,
    decay: f32,
    time: f32,
    delay_buffer: Vec<f32>,
    delay_pos: usize,
    offset: f32,
    drift: f32,
}

#[derive(Clone)]
struct Melody {
    osc: KarplusStrong,
    gain: f32,
    decay: f32,
    time: f32,
    vibrato: f32,
    delay_buffer: Vec<f32>,
    delay_pos: usize,
}

#[derive(Clone)]
struct Drum {
    osc: Oscillator,
    gain: f32,
    decay: f32,
    time: f32,
    filter: f32,
}