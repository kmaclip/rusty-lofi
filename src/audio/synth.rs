pub enum WaveType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

pub struct Oscillator {
    pub frequency: f32,
    pub phase: f32,
    pub phase_increment: f32,
    pub sample_rate: f32,
    pub wave_type: WaveType,
}

pub struct KarplusStrong {
    pub buffer: Vec<f32>,
    pub position: usize,
    pub frequency: f32,
    pub sample_rate: f32,
}

impl Oscillator {
    pub fn new(frequency: f32, sample_rate: f32, wave_type: WaveType) -> Self {
        let phase_increment = frequency / sample_rate;
        Self {
            frequency,
            phase: 0.0,
            phase_increment,
            sample_rate,
            wave_type,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = match self.wave_type {
            WaveType::Sine => (self.phase * 2.0 * std::f32::consts::PI).sin(),
            WaveType::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },
            WaveType::Sawtooth => 2.0 * self.phase - 1.0,
            WaveType::Triangle => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            },
            WaveType::Noise => rand::random::<f32>() * 2.0 - 1.0,
        };
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        sample
    }
}

impl KarplusStrong {
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        let buffer_size = (sample_rate / frequency).ceil() as usize;
        let mut buffer = Vec::with_capacity(buffer_size);
        
        // Fill buffer with random noise
        for _ in 0..buffer_size {
            buffer.push(rand::random::<f32>() * 2.0 - 1.0);
        }
        
        Self {
            buffer,
            position: 0,
            frequency,
            sample_rate,
        }
    }
    
    pub fn next_sample(&mut self, detune: f32) -> f32 {
        // Apply small random variations to the sample for a more organic sound
        let position = self.position;
        let next_position = (position + 1) % self.buffer.len();
        
        // Basic Karplus-Strong algorithm with slight variations
        let output = self.buffer[position];
        let next = self.buffer[next_position];
        
        // Update buffer with filtered value and detune
        let filtered = 0.5 * (output + next) * (0.996 - detune * 0.01);
        self.buffer[position] = filtered;
        self.position = next_position;
        
        output
    }
}