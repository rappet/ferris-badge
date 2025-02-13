#![no_std]
#![no_main]

mod button;
mod eye;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    low_power::{stop_ready, Executor, StopMode},
    peripherals::{IWDG, TIM21, TIM22},
    rcc::{self, MSIRange, VoltageScale},
    rtc::{Rtc, RtcConfig},
    time::Hertz,
    timer::{
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
    },
    wdg::IndependentWatchdog,
    Config,
};
use embassy_time::{Instant, Timer};
use panic_halt as _;
use static_cell::StaticCell;

use crate::{button::Button, eye::Eye};

#[cortex_m_rt::entry]
fn main() -> ! {
    Executor::take().run(|spawner| {
        spawner.spawn(async_main(spawner)).unwrap();
    });
}

#[embassy_executor::task]
async fn async_main(_spawner: Spawner) {
    let p = embassy_stm32::init({
        let mut config = Config::default();
        config.rcc = rcc::Config {
            msi: Some(MSIRange::RANGE4M),
            voltage_scale: VoltageScale::RANGE1,
            ..rcc::Config::default()
        };
        config.enable_debug_during_sleep = false;
        config
    });

    let rtc = Rtc::new(p.RTC, RtcConfig::default());
    static RTC: StaticCell<Rtc> = StaticCell::new();
    let rtc = RTC.init(rtc);
    embassy_stm32::low_power::stop_with_rtc(rtc);

    info!("Started 🦀");

    let mut btn_on = ExtiInput::new(p.PA9, p.EXTI9, Pull::Up);

    info!("Waiting for on!");
    info!("Low power mode enabled: {}", stop_ready(StopMode::Stop2));

    btn_on.wait_for_low().await;

    let mut good_boy = IndependentWatchdog::new(p.IWDG, 1_000 * 1_000 * 3);
    good_boy.unleash();

    good_boy.pet();

    info!("On!");

    let _anode_1 = Output::new(p.PA1, Level::High, Speed::Low);
    let _anode_2 = Output::new(p.PA5, Level::High, Speed::Low);

    let mut peripherals = {
        Timer::after_millis(1).await;

        let btn_on = Button::new(btn_on);
        let btn_mode = Button::new(Input::new(p.PA10, Pull::Up));

        let green_1 = PwmPin::new_ch1(p.PA2, OutputType::PushPull);
        let red_1 = PwmPin::new_ch2(p.PA3, OutputType::PushPull);
        let mut pwm_1 = SimplePwm::new(
            p.TIM21,
            Some(green_1),
            Some(red_1),
            None,
            None,
            Hertz::hz(500),
            CountingMode::EdgeAlignedUp,
        )
        .split();
        pwm_1.ch1.enable();
        pwm_1.ch2.enable();

        let eye_left = Eye::new(pwm_1.ch1, pwm_1.ch2);

        let green_2 = PwmPin::new_ch1(p.PA6, OutputType::PushPull);
        let red_2 = PwmPin::new_ch2(p.PA7, OutputType::PushPull);
        let mut pwm_2 = SimplePwm::new(
            p.TIM22,
            Some(green_2),
            Some(red_2),
            None,
            None,
            Hertz::hz(500),
            CountingMode::default(),
        )
        .split();
        pwm_2.ch1.enable();
        pwm_2.ch2.enable();

        let eye_right = Eye::new(pwm_2.ch1, pwm_2.ch2);

        BadgePeripherals {
            eye_left,
            eye_right,
            btn_on,
            btn_mode,
            good_boy,
        }
    };

    let mut mode = Mode::Red;

    loop {
        info!("Activated");

        active_loop(&mut peripherals, &mut mode).await;

        peripherals.eye_left.set_off();
        peripherals.eye_right.set_off();

        while !peripherals.btn_on.is_activated() {
            Timer::after_millis(10).await;
        }
    }
}

async fn active_loop(p: &mut BadgePeripherals<'_>, mode: &mut Mode) {
    let max_value = p.eye_left.max_value();
    info!("{}", max_value);

    while !p.btn_on.is_activated() {
        if p.btn_mode.is_activated() {
            mode.next();
        }
        match *mode {
            Mode::Red => {
                p.eye_left.set_red();
                p.eye_right.set_red();
            }
            Mode::Green => {
                p.eye_left.set_green();
                p.eye_right.set_green();
            }
            Mode::Yellow => {
                p.eye_left.set_value(max_value, max_value);
                p.eye_right.set_value(max_value, max_value);
            }
            Mode::Blink => {
                let now = Instant::now();
                if now.as_millis() % 1000 < 500 {
                    p.eye_left.set_red();
                    p.eye_right.set_green();
                } else {
                    p.eye_left.set_green();
                    p.eye_right.set_red();
                }
            }
            Mode::Fade => {
                let millis = (Instant::now().as_millis() % 4000) as u32;
                let fade = ((millis % 2000) * (max_value as u32) / 2000) as u16;
                if millis < 2000 {
                    p.eye_left.set_value(fade, max_value - fade);
                    p.eye_right.set_value(fade, max_value - fade);
                } else {
                    p.eye_left.set_value(max_value - fade, fade);
                    p.eye_right.set_value(max_value - fade, fade);
                }
            }
        }

        p.good_boy.pet();
        Timer::after_millis(1).await;
    }
    info!("Deactivated")
}

pub struct BadgePeripherals<'d> {
    pub eye_left: Eye<SimplePwmChannel<'d, TIM21>, SimplePwmChannel<'d, TIM21>>,
    pub eye_right: Eye<SimplePwmChannel<'d, TIM22>, SimplePwmChannel<'d, TIM22>>,
    pub btn_on: Button<ExtiInput<'d>>,
    pub btn_mode: Button<Input<'d>>,
    pub good_boy: IndependentWatchdog<'d, IWDG>,
}

pub enum Mode {
    Red,
    Green,
    Yellow,
    Blink,
    Fade,
}

impl Mode {
    pub fn next(&mut self) {
        *self = match *self {
            Mode::Red => Mode::Green,
            Mode::Green => Mode::Yellow,
            Mode::Yellow => Mode::Blink,
            Mode::Blink => Mode::Fade,
            Mode::Fade => Mode::Red,
        }
    }
}
