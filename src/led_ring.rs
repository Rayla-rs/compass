use blinksy::color::{ColorCorrection, Hsv, HsvHueRainbow};
use blinksy::drivers::ws2812::Ws2812Led;
use blinksy::layout::Layout1d;
use blinksy_esp::rmt::ClocklessRmtDriver;
use esp_hal::peripherals::*;
use esp_hal::rmt::{ConstChannelAccess, Rmt, Tx};
use esp_hal::time::Rate;

blinksy::layout1d!(RingLayout, 16);

const BUFFER_SIZE: usize = RingLayout::PIXEL_COUNT * 3 * 8 + 1;
pub struct LEDRing {
    driver: ClocklessRmtDriver<Ws2812Led, ConstChannelAccess<Tx, 0>, BUFFER_SIZE>,
}

impl LEDRing {
    pub fn new(rmt: esp_hal::peripherals::RMT<'static>, data_pin: GPIO2<'static>) -> Self {
        let rmt = Rmt::new(rmt, Rate::from_mhz(80)).unwrap();
        let rmt_channel = rmt.channel0;
        let driver = blinksy_esp::Ws2812Rmt::new(
            rmt_channel,
            data_pin,
            blinksy_esp::create_rmt_buffer!(RingLayout::PIXEL_COUNT, 3),
        );

        Self { driver }
    }

    fn fill(&mut self) {
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
