use crate::{display::Display, gps::Gps, hmc5883i::Hmc5883I, led_ring::LEDRing};

pub struct App {
    gps: Gps,
    hmc5883i: Hmc5883I,
    display_driver: Display,
    led_ring: LEDRing,
}

impl App {
    pub fn new(gps: Gps, hmc5883i: Hmc5883I, display_driver: Display, led_ring: LEDRing) -> Self {
        Self {
            gps,
            hmc5883i,
            display_driver,
            led_ring,
        }
    }
    pub async fn run(mut self) -> ! {
        loop {
            match self.gps.process().await {
                Ok(_) => {}
                Err(_) => {}
            }

            if let Err(_) = self.hmc5883i.process().await {
                // Handle i2c err by clearing the state of the magnetometer
                self.hmc5883i.clear();
            }
        }
    }
}
