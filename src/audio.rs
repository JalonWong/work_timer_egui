use rodio::OutputStreamBuilder;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
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

    pub fn schedule_notify(&mut self, name: impl AsRef<Path>, after_secs: u64) {
        let name = if let Ok(true) = fs::exists(name.as_ref()) {
            name.as_ref().to_path_buf()
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

            let stream_handle = OutputStreamBuilder::open_default_stream().unwrap();
            let file = BufReader::new(File::open(name).unwrap());
            let sink = rodio::play(stream_handle.mixer(), file).unwrap();

            while !sink.empty() {
                thread::park_timeout(Duration::from_millis(100));
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
