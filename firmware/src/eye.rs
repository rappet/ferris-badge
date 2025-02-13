use embedded_hal::pwm::SetDutyCycle;

pub struct Eye<A: SetDutyCycle, B: SetDutyCycle>(A, B);

impl<A: SetDutyCycle, B: SetDutyCycle> Eye<A, B> {
    pub fn new(pwm_red: A, pwm_green: B) -> Self {
        let mut eye = Self(pwm_red, pwm_green);
        eye.set_off();
        eye
    }

    pub fn set_off(&mut self) {
        self.0.set_duty_cycle(self.0.max_duty_cycle()).ok();
        self.1.set_duty_cycle(self.1.max_duty_cycle()).ok();
    }

    pub fn set_green(&mut self) {
        self.set_value(0, self.max_value());
    }

    pub fn set_red(&mut self) {
        self.set_value(self.max_value(), 0);
    }

    pub fn set_value(&mut self, red: u16, green: u16) {
        self.0.set_duty_cycle(self.max_value() - green).ok();
        // red is _really_ bright
        self.1.set_duty_cycle(self.max_value() - (red >> 3)).ok();
    }

    pub fn max_value(&self) -> u16 {
        self.0.max_duty_cycle()
    }
}
