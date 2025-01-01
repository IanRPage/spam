use crate::data::system::System;
use std::{thread, time::Duration};

pub struct App {
    system: System,
}

impl App {
    pub fn new() -> App {
        App {
            system: System::new(),
        }
    }

    pub fn run(&mut self) {
        let sys = &mut self.system;
        loop {
            sys.update_sys();
            sys.display();
            thread::sleep(Duration::from_millis(200));
        }
    }
}
