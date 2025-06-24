use rodio::Sink;
use rodio::{Decoder, OutputStream};
use std::fs::{self, File};
use std::io::BufReader;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Audio {
    play_th: Option<JoinHandle<()>>,
    run_flag: Arc<AtomicBool>,
}

impl Audio {
    pub fn new() -> Self {
        Self {
            play_th: None,
            run_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn schedule_notify(&mut self, name: &str, after_secs: u64) {
        let name = if let Ok(true) = fs::exists(name) {
            name.to_string()
        } else {
            return;
        };

        self.run_flag.store(true, Ordering::SeqCst);
        let run_flag = Arc::clone(&self.run_flag);

        self.play_th = Some(thread::spawn(move || {
            thread::park_timeout(Duration::from_secs(after_secs));
            if !run_flag.load(Ordering::SeqCst) {
                return;
            }

            // _stream must live as long as the sink
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            // Load a sound from a file, using a path relative to Cargo.toml
            let file = BufReader::new(File::open(name).unwrap());
            // Decode that sound file into a source
            let source = Decoder::new(file).unwrap();
            sink.append(source);

            while !sink.empty() {
                thread::park_timeout(Duration::from_millis(300));
                if !run_flag.load(Ordering::SeqCst) {
                    break;
                }
            }

            sink.stop();
        }));
    }

    pub fn cancel_notify(&mut self) {
        if let Some(th) = self.play_th.take() {
            self.run_flag.store(false, Ordering::SeqCst);
            th.thread().unpark();
            th.join().unwrap();
        }
    }
}
