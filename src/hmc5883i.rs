use esp_hal::{
    i2c::master::{Config, Error, I2c},
    peripherals::*,
    Async,
};

pub struct Hmc5883I {
    i2c_port: I2c<'static, Async>,
    x_guass: f32,
    y_guass: f32,
    z_guass: f32,
}

impl Hmc5883I {
    pub fn new(i2c: I2C0<'static>) -> Self {
        let config = Config::default();
        let i2c_port = I2c::new(i2c, config).unwrap().into_async();

        Self {
            i2c_port,
            x_guass: f32::NAN,
            y_guass: f32::NAN,
            z_guass: f32::NAN,
        }
    }

    /// Calculates heading in radians
    fn heading(&mut self) -> f32 {
        micromath::F32::atan2(self.y_guass.into(), self.x_guass.into()).into()
    }

    pub async fn process(&mut self) -> Result<(), Error> {
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

        self.x_guass = (raw[0] as f32) * resolution;
        self.y_guass = (raw[1] as f32) * resolution;
        self.z_guass = (raw[2] as f32) * resolution;

        Ok(())
    }

    pub fn clear(&mut self) {
        self.x_guass = f32::NAN;
        self.y_guass = f32::NAN;
        self.z_guass = f32::NAN;
    }
}
