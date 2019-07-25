use rodio::Sink;
use std::cmp::max;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::sync::mpsc;
use lazy_static;

lazy_static! {
    static ref DEVICE: rodio::Device = rodio::default_output_device().unwrap();
}

pub struct Player {
    tx: Option<mpsc::Sender<String>>,
    stopped: bool,
    paused: bool,
}

impl Player {
    pub fn new() -> Self {
        Player {
            tx: None,
            stopped: false,
            paused: false,
        }
    }

    pub fn pause(&mut self) {
        if !self.paused {
            if let Some(t) = &self.tx {
                t.send("pause".to_string()).unwrap();
            }
            self.paused = true;
        }
    }

    pub fn stop(&mut self) {

        if !self.stopped {
            if let Some(t) = &self.tx {
                t.send("stop".to_string()).unwrap();
            }
        }
        self.stopped = true;
        self.paused = false;
    }

    pub fn play(&mut self, path: String) {
        if !self.paused {
            let (tx, rx) = mpsc::channel();
            self.tx = Some(tx);
            self.stopped = false;
            thread::spawn(move || {
                let sink = Sink::new(&DEVICE);
                let file = File::open(&path).unwrap();
                let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                sink.append(source);

                loop {
                    if let Ok(c) = rx.try_recv() {
                        match &c[..] {
                            "stop" => break,
                            "pause" => sink.pause(),
                            "unpause" => sink.play(),
                            _ => (),
                        }
                    }
                }

                sink.stop();
                sink.detach();
            });
        } else {
            if let Some(t) = &self.tx {
                t.send("unpause".to_string()).unwrap();
            }
            self.paused = false;
        }
    }
}