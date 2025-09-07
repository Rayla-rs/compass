use core::cell::Cell;

use critical_section::Mutex;
use embassy_time::{Duration, Ticker};
use esp_hal::{
    i2c::master::{Config, Error, I2c},
    peripherals::*,
    Async,
};

#[embassy_executor::task]
pub async fn magnetometer_task(
    i2c: I2C0<'static>,
    state: &'static Mutex<Cell<MagnetometerState>>,
) -> ! {
    let mut hmc5883i = Hmc5883I::new(i2c, state);
    let mut ticker = Ticker::every(Duration::from_millis(50));

    loop {
        hmc5883i.process().await;
        ticker.next().await;
    }
}

#[derive(Clone, Copy)]
pub struct MagnetometerState {
    x_guass: f32,
    y_guass: f32,
    z_guass: f32,
}

impl Default for MagnetometerState {
    fn default() -> Self {
        Self::new()
    }
}

impl MagnetometerState {
    pub const fn new() -> Self {
        Self {
            x_guass: f32::NAN,
            y_guass: f32::NAN,
            z_guass: f32::NAN,
        }
    }
    /// Calculates heading in radians
    fn _heading(&mut self) -> f32 {
        micromath::F32::atan2(self.y_guass.into(), self.x_guass.into()).into()
    }
}

struct Hmc5883I {
    i2c_port: I2c<'static, Async>,
    state: &'static Mutex<Cell<MagnetometerState>>,
}

impl Hmc5883I {
    fn new(i2c: I2C0<'static>, state: &'static Mutex<Cell<MagnetometerState>>) -> Self {
        let config = Config::default();
        let i2c_port = I2c::new(i2c, config).unwrap().into_async();

        Self { i2c_port, state }
    }

    async fn process(&mut self) {
        let result = self.process_inner().await;

        critical_section::with(|cs| self.state.borrow(cs).set(result.unwrap_or_default()));
    }

    async fn process_inner(&mut self) -> Result<MagnetometerState, Error> {
        let mut buf = [0u8; 6];
        let mut gain = [0u8; 1];

        self.i2c_port.read_async(1, &mut gain[0..=0]).await?;
        let gain = gain[0] >> 5;
        let resolution: f32 = match gain {
            0x00 => 0.73,
            0x01 => 0.92,
            0x02 => 1.22,
            0x03 => 1.52,
            0x04 => 2.27,
            0x05 => 2.56,
            0x06 => 3.03,
            0x07 => 4.35,
            _ => 0.0,
        };

        self.i2c_port.read_async(0x03, &mut buf[0..=0]).await?; // x
        self.i2c_port.read_async(0x04, &mut buf[1..=1]).await?;
        self.i2c_port.read_async(0x05, &mut buf[2..=2]).await?; // z
        self.i2c_port.read_async(0x06, &mut buf[3..=3]).await?;
        self.i2c_port.read_async(0x07, &mut buf[4..=4]).await?; // y
        self.i2c_port.read_async(0x08, &mut buf[5..=5]).await?;

        let raw = [
            ((buf[0] as i16) << 8) | buf[1] as i16,
            ((buf[4] as i16) << 8) | buf[5] as i16,
            ((buf[2] as i16) << 8) | buf[3] as i16,
        ];

        Ok(MagnetometerState {
            x_guass: (raw[0] as f32) * resolution,
            y_guass: (raw[1] as f32) * resolution,
            z_guass: (raw[2] as f32) * resolution,
        })
    }
}
