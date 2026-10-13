//! Short synthesized tones, for hearing an algorithm as well as seeing it.
//!
//! The browser build uses the Web Audio API. The desktop build uses `cpal`
//! when compiled with `--features native-audio`; without it, [`Audio`] is
//! silent and reports that sound is unavailable.

/// Lowest and highest pitch, in Hz.
pub const LOW_HZ: f32 = 150.0;
pub const HIGH_HZ: f32 = 800.0;

/// `150 + X_norm · 650`: small values sound low, large values high.
pub fn frequency(norm: f32) -> f32 {
    LOW_HZ + norm.clamp(0.0, 1.0) * (HIGH_HZ - LOW_HZ)
}

/// Plays tones. Silent until [`Audio::set_enabled`] turns it on, which
/// browsers require to happen in response to a click.
pub struct Audio {
    enabled: bool,
    /// 0..=1
    pub volume: f32,
    backend: Option<backend::Backend>,
    /// Set when the backend failed to start, so we stop retrying.
    failed: bool,
}

impl Default for Audio {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 0.25,
            backend: None,
            failed: false,
        }
    }
}

impl Audio {
    /// Whether this build can make sound at all.
    pub fn available() -> bool {
        backend::AVAILABLE
    }

    pub fn enabled(&self) -> bool {
        self.enabled && !self.failed
    }

    pub fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
        if on && self.backend.is_none() && !self.failed {
            match backend::Backend::new() {
                Some(b) => self.backend = Some(b),
                None => self.failed = true,
            }
        }
        if let Some(b) = &self.backend {
            b.resume();
        }
    }

    /// Why sound is off, if it cannot be turned on.
    pub fn unavailable_reason(&self) -> Option<&'static str> {
        if !Self::available() {
            Some("build with --features native-audio for sound on desktop")
        } else if self.failed {
            Some("no audio output device")
        } else {
            None
        }
    }

    /// Play a tone at `hz` that fades out over `seconds`.
    pub fn tone(&mut self, hz: f32, seconds: f32) {
        if !self.enabled() || self.volume <= 0.0 {
            return;
        }
        if let Some(b) = &mut self.backend {
            b.tone(hz, seconds, self.volume);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod backend {
    use web_sys::{AudioContext, OscillatorType};

    pub const AVAILABLE: bool = true;

    pub struct Backend {
        ctx: AudioContext,
    }

    impl Backend {
        pub fn new() -> Option<Self> {
            AudioContext::new().ok().map(|ctx| Self { ctx })
        }

        pub fn resume(&self) {
            let _ = self.ctx.resume();
        }

        pub fn tone(&mut self, hz: f32, seconds: f32, volume: f32) {
            let _ = self.try_tone(hz, seconds, volume);
        }

        fn try_tone(
            &self,
            hz: f32,
            seconds: f32,
            volume: f32,
        ) -> Result<(), wasm_bindgen::JsValue> {
            let t = self.ctx.current_time();
            let osc = self.ctx.create_oscillator()?;
            osc.set_type(OscillatorType::Triangle);
            osc.frequency().set_value(hz);
            let gain = self.ctx.create_gain()?;
            gain.gain().set_value_at_time(volume * 0.6, t)?;
            gain.gain()
                .exponential_ramp_to_value_at_time(0.0001, t + seconds as f64)?;
            osc.connect_with_audio_node(&gain)?;
            gain.connect_with_audio_node(&self.ctx.destination())?;
            osc.start()?;
            osc.stop_with_when(t + seconds as f64 + 0.02)?;
            Ok(())
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-audio"))]
mod backend {
    use std::sync::{Arc, Mutex};

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    pub const AVAILABLE: bool = true;
    /// More simultaneous tones than this and the oldest are dropped.
    const MAX_VOICES: usize = 12;

    struct Voice {
        hz: f32,
        phase: f32,
        left: u32,
        total: u32,
        amp: f32,
    }

    pub struct Backend {
        voices: Arc<Mutex<Vec<Voice>>>,
        rate: f32,
        _stream: cpal::Stream,
    }

    /// A triangle wave: softer than a square, brighter than a sine.
    fn triangle(phase: f32) -> f32 {
        4.0 * (phase - (phase + 0.5).floor()).abs() - 1.0
    }

    fn mix(voices: &mut Vec<Voice>, rate: f32) -> f32 {
        let mut out = 0.0;
        for v in voices.iter_mut() {
            let envelope = v.left as f32 / v.total as f32;
            out += triangle(v.phase) * v.amp * envelope * envelope;
            v.phase = (v.phase + v.hz / rate).fract();
            v.left -= 1;
        }
        voices.retain(|v| v.left > 0);
        out.clamp(-1.0, 1.0)
    }

    impl Backend {
        pub fn new() -> Option<Self> {
            let device = cpal::default_host().default_output_device()?;
            let config = device.default_output_config().ok()?;
            let format = config.sample_format();
            let config: cpal::StreamConfig = config.into();
            let rate = config.sample_rate as f32;
            let channels = config.channels as usize;
            let voices: Arc<Mutex<Vec<Voice>>> = Arc::default();
            let shared = voices.clone();
            let on_error = |e| eprintln!("audio stream error: {e}");
            let stream = match format {
                cpal::SampleFormat::F32 => device
                    .build_output_stream(
                        config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            let mut voices = shared.lock().unwrap();
                            for frame in data.chunks_mut(channels) {
                                let s = mix(&mut voices, rate);
                                frame.fill(s);
                            }
                        },
                        on_error,
                        None,
                    )
                    .ok()?,
                cpal::SampleFormat::I16 => device
                    .build_output_stream(
                        config,
                        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                            let mut voices = shared.lock().unwrap();
                            for frame in data.chunks_mut(channels) {
                                let s = (mix(&mut voices, rate) * i16::MAX as f32) as i16;
                                frame.fill(s);
                            }
                        },
                        on_error,
                        None,
                    )
                    .ok()?,
                _ => return None,
            };
            stream.play().ok()?;
            Some(Self {
                voices,
                rate,
                _stream: stream,
            })
        }

        pub fn resume(&self) {}

        pub fn tone(&mut self, hz: f32, seconds: f32, volume: f32) {
            let total = ((seconds * self.rate) as u32).max(1);
            let mut voices = self.voices.lock().unwrap();
            if voices.len() >= MAX_VOICES {
                voices.remove(0);
            }
            voices.push(Voice {
                hz,
                phase: 0.0,
                left: total,
                total,
                amp: volume * 0.5,
            });
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-audio")))]
mod backend {
    pub const AVAILABLE: bool = false;

    pub struct Backend;

    impl Backend {
        pub fn new() -> Option<Self> {
            None
        }

        pub fn resume(&self) {}

        pub fn tone(&mut self, _hz: f32, _seconds: f32, _volume: f32) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_mapping() {
        assert_eq!(frequency(0.0), 150.0);
        assert_eq!(frequency(1.0), 800.0);
        assert_eq!(frequency(0.5), 475.0);
        assert_eq!(frequency(-1.0), 150.0);
        assert_eq!(frequency(2.0), 800.0);
    }

    #[test]
    fn silent_until_enabled() {
        let mut a = Audio::default();
        assert!(!a.enabled());
        a.tone(440.0, 0.1); // must not panic or play
    }
}
