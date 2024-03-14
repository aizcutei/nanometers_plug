use bytemuck;
use interprocess::local_socket::{LocalSocketListener, NameTypeSupport};
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::{io::prelude::*, sync::Arc};
mod editor;

#[cfg(target_os = "macos")]
pub const RING_BUFFER_SIZE: usize = 44100;

#[cfg(not(target_os = "macos"))]
pub const RING_BUFFER_SIZE: usize = 48000;

pub struct NanometersPlug {
    params: Arc<NanometersPlugParams>,
    listener: Arc<LocalSocketListener>,
    ring_buffer: Vec<f32>,
}

#[derive(Params)]
struct NanometersPlugParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
}

impl Default for NanometersPlug {
    fn default() -> Self {
        let name = {
            use NameTypeSupport::*;
            match NameTypeSupport::query() {
                OnlyPaths => "/tmp/nanometers.sock",
                OnlyNamespaced | Both => "@nanometers.sock",
            }
        };

        #[cfg(not(target_os = "windows"))]
        {
            use std::fs;
            if fs::metadata(&name).is_ok() {
                fs::remove_file(&name).expect("ERR: failed to remove old socket");
            }
        }

        let listener = LocalSocketListener::bind(name).expect("ERR: failed to bind to socket");

        listener
            .set_nonblocking(true)
            .expect("ERR: failed to set nonblocking");

        Self {
            params: Arc::new(NanometersPlugParams::default()),
            listener: Arc::new(listener),
            ring_buffer: vec![0.0; RING_BUFFER_SIZE + 1],
        }
    }
}

impl Default for NanometersPlugParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
        }
    }
}

impl Plugin for NanometersPlug {
    const NAME: &'static str = "Nanometers Server";
    const VENDOR: &'static str = "aizcutei";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "aiz.cutei@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Get current buffer
        let temp_buffer = buffer.as_slice().concat();

        // Update Ring buffer
        let ring_buffer_index = self.ring_buffer[0] as usize;
        if ring_buffer_index + temp_buffer.len() > RING_BUFFER_SIZE {
            let split_index = RING_BUFFER_SIZE - ring_buffer_index;
            let (first, second) = temp_buffer.split_at(split_index);
            self.ring_buffer[ring_buffer_index + 1..].copy_from_slice(&first[..]);
            self.ring_buffer[1..second.len() + 1].copy_from_slice(&second[..]);
            self.ring_buffer[0] = second.len() as f32;
        } else {
            self.ring_buffer[ring_buffer_index + 1..ring_buffer_index + 1 + temp_buffer.len()]
                .copy_from_slice(&temp_buffer[..]);
            self.ring_buffer[0] += temp_buffer.len() as f32;
        }

        // Send buffer to client
        let mut conn = match self.listener.accept() {
            Ok(conn) => conn,
            Err(_) => return ProcessStatus::Normal,
        };
        let send_bytes = bytemuck::cast_slice(&self.ring_buffer);
        conn.write(send_bytes).unwrap();
        conn.flush().unwrap();

        ProcessStatus::Normal
    }
}

impl ClapPlugin for NanometersPlug {
    const CLAP_ID: &'static str = "com.aizcutei.nanometers-plug";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Plugin server for nanometers.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Utility];
}

impl Vst3Plugin for NanometersPlug {
    const VST3_CLASS_ID: [u8; 16] = *b"NANOMETERSSERVER";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(NanometersPlug);
nih_export_vst3!(NanometersPlug);
