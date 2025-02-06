use embassy_stm32::timer::{simple_pwm::SimplePwmChannel, GeneralInstance4Channel};

pub struct Eye<'a, 'b, A: GeneralInstance4Channel, B: GeneralInstance4Channel>(
    SimplePwmChannel<'a, A>,
    SimplePwmChannel<'b, B>,
);

impl<'a, 'b, A: GeneralInstance4Channel, B: GeneralInstance4Channel> Eye<'a, 'b, A, B> {
    pub fn new(pwm_red: SimplePwmChannel<'a, A>, pwm_green: SimplePwmChannel<'b, B>) -> Self {
        let mut eye = Self(pwm_red, pwm_green);
        eye.set_off();
        eye.0.enable();
        eye.1.enable();
        eye
    }

    pub fn set_off(&mut self) {
        self.0.set_duty_cycle(self.0.max_duty_cycle());
        self.1.set_duty_cycle(self.1.max_duty_cycle());
    }

    pub fn set_green(&mut self) {
        self.set_value(0, self.max_value());
    }

    pub fn set_red(&mut self) {
        self.set_value(self.max_value(), 0);
    }

    pub fn set_value(&mut self, red: u16, green: u16) {
        self.0.set_duty_cycle(self.max_value() - green);
        // red is _really_ bright
        self.1.set_duty_cycle(self.max_value() - (red >> 3));
    }

    pub fn max_value(&self) -> u16 {
        self.0.max_duty_cycle()
    }
}
