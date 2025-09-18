use core::cell::Cell;
use core::f32::consts::PI;

use blinksy::color::{ColorCorrection, LinearSrgb};
use blinksy::drivers::ws2812::Ws2812Led;
use blinksy::layout::Layout1d;
use blinksy_esp::rmt::ClocklessRmtDriver;

use critical_section::Mutex;

use embassy_time::{Duration, Ticker};

use esp_hal::peripherals::*;
use esp_hal::rmt::{ConstChannelAccess, Rmt, Tx};
use esp_hal::time::Rate;

use crate::compass::CompassState;
use crate::gps::NavPvtState;

#[embassy_executor::task]
pub async fn led_ring_task(
    rmt: esp_hal::peripherals::RMT<'static>,
    data_pin: GPIO2<'static>,
    _nav_pvt_state: &'static Mutex<Cell<NavPvtState>>,
    _magnetometer_state: &'static Mutex<Cell<CompassState>>,
) -> ! {
    let mut ring = LEDRing::new(rmt, data_pin);
    let mut ticker = Ticker::every(Duration::from_millis(50));

    loop {
        // UNFINISHED
        let arg = critical_section::with(|_cs| {
            //
            ProcessArgument::NotReady
        });

        ring.process(arg);
        ticker.next().await;
    }
}

enum ProcessArgument {
    WithNorth(usize),
    WithNorthAndTarget(usize, usize),
    NotReady,
}

blinksy::layout1d!(RingLayout, 16);

const BUFFER_SIZE: usize = RingLayout::PIXEL_COUNT * 3 * 8 + 1;

fn radians_to_pixel(rad: f32) -> usize {
    Into::<f32>::into(micromath::F32(rad / PI * 8.).round()) as usize % RingLayout::PIXEL_COUNT
}

struct LEDRing {
    driver: ClocklessRmtDriver<Ws2812Led, ConstChannelAccess<Tx, 0>, BUFFER_SIZE>,
}

impl LEDRing {
    fn new(rmt: esp_hal::peripherals::RMT<'static>, data_pin: GPIO2<'static>) -> Self {
        let rmt = Rmt::new(rmt, Rate::from_hz(400)).unwrap();
        let rmt_channel = rmt.channel0;
        let driver = blinksy_esp::Ws2812Rmt::new(
            rmt_channel,
            data_pin,
            blinksy_esp::create_rmt_buffer!(RingLayout::PIXEL_COUNT, 3),
        );

        Self { driver }
    }

    fn process(&mut self, _arg: ProcessArgument) {
        self.driver
            .write_pixels(
                RingLayout::points()
                    .into_iter()
                    .map(|_| LinearSrgb::new(0., 1., 0.)),
                1f32,
                ColorCorrection::default(),
            )
            .unwrap();
    }
}
