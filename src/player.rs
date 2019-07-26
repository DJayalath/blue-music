use lazy_static;
use rodio::Sink;
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;

lazy_static! {
    static ref DEVICE: rodio::Device = rodio::default_output_device().unwrap();
}

enum PlayerState {
    Stop,
    Pause,
    Unpause,
}

pub struct Player {
    tx: Option<mpsc::Sender<PlayerState>>,
    stopped: bool,
    paused: bool,
    pub duration: u128,
}

impl Player {
    pub fn new() -> Self {
        Player {
            tx: None,
            stopped: false,
            paused: false,
            duration: 0,
        }
    }

    pub fn pause(&mut self) {
        if let Some(t) = &self.tx {
            if let Err(e) = t.send(PlayerState::Pause) {
                println!("WARN: Attempted to pause non-existant playing thread!\n{}", e);
            }
        }
        self.paused = true;
    }

    pub fn stop(&mut self) {
        if let Some(t) = &self.tx {
            if let Err(e) = t.send(PlayerState::Stop) {
                println!("WARN: Attempted to stop non-existant playing thread!\n{}", e);
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
            let file = File::open(&path).unwrap();
            let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
            self.duration = source.total_duration().unwrap().as_millis();
            thread::spawn(move || {
                let sink = Sink::new(&DEVICE);
                sink.append(source);
                loop {
                    if let Ok(c) = rx.try_recv() {
                        match c {
                            PlayerState::Stop => break,
                            PlayerState::Pause => sink.pause(),
                            PlayerState::Unpause => sink.play(),
                        }
                    }
                }

                sink.stop();
                sink.detach();
            });
        } else {
            if let Some(t) = &self.tx {
                if let Err(e) = t.send(PlayerState::Unpause) {
                    println!("WARN: Attempted to unpause non-existant playing thread!\n{}", e);
                }
            }
            self.paused = false;
        }
    }
}
