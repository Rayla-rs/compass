use core::cell::Cell;

use blinksy::color::{ColorCorrection, Hsv, HsvHueRainbow};
use blinksy::drivers::ws2812::Ws2812Led;
use blinksy::layout::Layout1d;
use blinksy_esp::rmt::ClocklessRmtDriver;
use critical_section::Mutex;
use embassy_time::{Duration, Ticker};
use esp_hal::peripherals::*;
use esp_hal::rmt::{ConstChannelAccess, Rmt, Tx};
use esp_hal::time::Rate;

use crate::gps::NavPvtState;
use crate::hmc5883i::MagnetometerState;

#[embassy_executor::task]
pub async fn led_ring_task(
    rmt: esp_hal::peripherals::RMT<'static>,
    data_pin: GPIO2<'static>,
    _nav_pvt_state: Mutex<Cell<NavPvtState>>,
    _magnetometer_state: Mutex<Cell<MagnetometerState>>,
) -> ! {
    let mut ring = LEDRing::new(rmt, data_pin);
    let mut ticker = Ticker::every(Duration::from_millis(200));

    loop {
        // TODO
        // Create update args in critical section

        ring.process();
        ticker.next().await;
    }
}

blinksy::layout1d!(RingLayout, 16);

const BUFFER_SIZE: usize = RingLayout::PIXEL_COUNT * 3 * 8 + 1;
struct LEDRing {
    driver: ClocklessRmtDriver<Ws2812Led, ConstChannelAccess<Tx, 0>, BUFFER_SIZE>,
}

impl LEDRing {
    fn new(rmt: esp_hal::peripherals::RMT<'static>, data_pin: GPIO2<'static>) -> Self {
        let rmt = Rmt::new(rmt, Rate::from_mhz(80)).unwrap();
        let rmt_channel = rmt.channel0;
        let driver = blinksy_esp::Ws2812Rmt::new(
            rmt_channel,
            data_pin,
            blinksy_esp::create_rmt_buffer!(RingLayout::PIXEL_COUNT, 3),
        );

        Self { driver }
    }

    fn process(&mut self) {
        self.driver
            .write_pixels(
                RingLayout::points()
                    .into_iter()
                    .map(|_| Hsv::<HsvHueRainbow>::new(1., 1., 1.)),
                1f32,
                ColorCorrection::default(),
            )
            .unwrap();
    }
}
