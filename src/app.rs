use crate::{device::Device, display::Display, led_ring::LEDRing};

pub struct App {
    device: Device,
    display_driver: Display,
    led_ring: LEDRing,
}

impl App {
    pub async fn run(self) -> ! {
        loop {}
    }
}
