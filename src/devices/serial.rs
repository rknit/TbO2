use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread::{self},
};

use crate::Device;

pub struct SerialIO<S> {
    s: Arc<Mutex<S>>,
    detached: Arc<Mutex<bool>>,
    read_keys: Vec<u8>,
    display_keys: Arc<Mutex<Vec<u8>>>,
}
impl<S: Write + Read + Send + 'static> SerialIO<S> {
    pub fn new(s: S) -> Self {
        Self {
            s: Arc::new(Mutex::new(s)),
            detached: Arc::new(Mutex::new(true)),
            read_keys: vec![],
            display_keys: Arc::new(Mutex::new(vec![])),
        }
    }
}
impl<S: Write + Read + Send + 'static> Device for SerialIO<S> {
    fn reset(&mut self) {
        self.read_keys.clear();
        self.display_keys.lock().unwrap().clear();
    }

    fn attach(&mut self) {
        {
            *self.detached.lock().unwrap() = false;
        }

        let dt = self.detached.clone();
        let swr = self.s.clone();
        let dk = self.display_keys.clone();
        thread::spawn(move || {
            while {
                let dt = dt.lock().unwrap();
                !*dt
            } {
                let mut dk = dk.lock().unwrap();
                if dk.is_empty() {
                    continue;
                }
                let mut swr = swr.lock().unwrap();
                if let Ok(n) = swr.write(&dk) {
                    dk.drain(0..n);
                };
            }
        });
    }

    fn detach(&mut self) {
        *self.detached.lock().unwrap() = true;
    }

    fn read(&mut self, _: usize) -> Option<u8> {
        if self.read_keys.is_empty() {
            {
                let mut s = self.s.lock().unwrap();
                s.read_to_end(&mut self.read_keys).ok()?;
            }
            self.read(0)
        } else {
            Some(self.read_keys.remove(0))
        }
    }

    fn write(&mut self, _: usize, data: u8) -> Option<()> {
        self.display_keys.lock().unwrap().push(data);
        Some(())
    }
}
