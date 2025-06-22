use rodio::{Decoder, OutputStream, source::Source};
use std::fs::{self, File};
use std::io::BufReader;

pub struct Audio {
    stream: Option<OutputStream>,
}

impl Audio {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn play_notify(&mut self, name: Option<&str>) {
        let name = if let Some(name) = name {
            if let Ok(true) = fs::exists(name) {
                name
            } else {
                return;
            }
        } else {
            return;
        };

        // Get an output stream handle to the default physical sound device.
        // Note that no sound will be played if _stream is dropped
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        self.stream.replace(stream);
        // Load a sound from a file, using a path relative to Cargo.toml
        let file = BufReader::new(File::open(name).unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();
        // Play the sound directly on the device
        stream_handle.play_raw(source.convert_samples()).unwrap();
    }
}
