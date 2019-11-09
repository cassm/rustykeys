use portaudio as pa;
use std::f64::consts::PI;

pub struct Synth {
    pub pa: pa::PortAudio,
}

impl Synth {
    pub fn play_note(&self, freq: f64, duration_millis: i32, verbose: bool) -> Result<(), pa::Error> {
        const CHANNELS: i32 = 2;
        const FRAMES_PER_BUFFER: u32 = 64;
        const TABLE_SIZE: usize = 200;
        const SAMPLE_RATE: f64 = 64000.0;

        // Initialise sinusoidal wavetable.
        let mut sine = [0.0; TABLE_SIZE];

        for i in 0..TABLE_SIZE {
            sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;
        }

        let mut settings = self.pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
        // we won't output out of range samples so don't bother clipping them.
        settings.flags = pa::stream_flags::CLIP_OFF;

        // This routine will be called by the PortAudio engine when audio is needed. It may called at
        // interrupt level on some machines so don't do anything that could mess up the system like
        // dynamic resource allocation or IO.
        let increment: f64 = (TABLE_SIZE as f64 / SAMPLE_RATE) * freq;
        let mut left_phase = 0.0;
        let mut right_phase = 0.0;

        let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
            let mut idx = 0;
            for _ in 0..frames {
                buffer[idx] = sine[left_phase as usize];
                buffer[idx + 1] = sine[right_phase as usize];
                left_phase += increment;
                if left_phase as usize >= TABLE_SIZE {
                    left_phase -= TABLE_SIZE as f64;
                }
                right_phase += increment;
                if right_phase as usize >= TABLE_SIZE {
                    right_phase -= TABLE_SIZE as f64;
                }
                idx += 2;
            }
            pa::Continue
        };

        let mut stream = self.pa.open_non_blocking_stream(settings, callback)?;

        stream.start()?;

        if verbose {
            println!("Play {}Hz for {} milliseconds.", freq, duration_millis);
        }

        self.pa.sleep(duration_millis);

        stream.stop()?;
        stream.close()?;

        Ok(())
    }
}
