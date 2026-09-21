use std::time::Duration;
use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

#[allow(dead_code)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Resize(u16, u16),
}

pub struct EventHandler {
    receiver: std::sync::mpsc::Receiver<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            let tick_rate = Duration::from_millis(tick_rate);
            let mut last_tick = std::time::Instant::now();
            
            loop {
                let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(0));
                
                if event::poll(timeout).expect("failed to poll event") {
                    match event::read().expect("failed to read event") {
                        CrosstermEvent::Key(e) => {
                            if e.kind == event::KeyEventKind::Press {
                                sender.send(Event::Key(e)).expect("failed to send key event");
                            }
                        }
                        CrosstermEvent::Resize(w, h) => {
                            sender.send(Event::Resize(w, h)).expect("failed to send resize event");
                        }
                        _ => {}
                    }
                }
                
                if last_tick.elapsed() >= tick_rate {
                    sender.send(Event::Tick).expect("failed to send tick event");
                    last_tick = std::time::Instant::now();
                }
            }
        });
        
        Self { receiver }
    }

    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}
