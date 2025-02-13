use defmt::{unwrap, warn};
use embassy_time::{Duration, Instant};
use embedded_hal::digital::InputPin;

pub struct Button<I: InputPin> {
    input: I,
    last_state: bool,
    last_activation: Instant,
}

impl<I: InputPin> Button<I> {
    pub fn new(mut input: I) -> Self {
        let last_state = unwrap!(input.is_low().ok());
        let last_switch = Instant::now();
        Self {
            input,
            last_state,
            last_activation: last_switch,
        }
    }

    pub fn is_activated(&mut self) -> bool {
        let new_state = unwrap!(self.input.is_low().ok());
        let now = Instant::now();

        if now.duration_since(self.last_activation) > Duration::from_millis(250) {
            if self.last_state && !new_state {
                self.last_state = false;
                false
            } else if !self.last_state && new_state {
                self.last_state = true;
                self.last_activation = now;
                warn!("PUSH");
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
