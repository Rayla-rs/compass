use core::cell::Cell;

use crate::qmc5883l::QMC5883L;
use critical_section::Mutex;
use embassy_time::{Duration, Ticker};
use esp_hal::{
    i2c::master::{Config, I2c},
    peripherals::*,
};
use esp_println::println;

#[derive(Debug, Clone, Copy)]
pub struct CompassState {
    pub temp: i16,
    pub mag: (i16, i16, i16),
}

pub struct NavCompassState {
    pub temp: i16,
    pub north_dir: f32,
    pub target_dir: f32,
    pub screen_offset: f32,
}

pub static COMPASS_STATE: Mutex<Cell<CompassState>> = Mutex::new(Cell::new(CompassState {
    temp: 0,
    mag: (0, 0, 0),
}));

#[embassy_executor::task]
pub async fn compass_task(i2c: I2C0<'static>, sda: GPIO22<'static>, scl: GPIO23<'static>) -> ! {
    println!("Started Compass Task");

    let config = Config::default();
    let i2c = I2c::new(i2c, config)
        .unwrap()
        .with_sda(sda)
        .with_scl(scl)
        .into_async();

    let mut qmc5883l = QMC5883L::new(i2c).unwrap();
    qmc5883l.continuous().unwrap();

    let mut ticker = Ticker::every(Duration::from_millis(1000));

    loop {
        if let Ok(mag) = qmc5883l.mag() {
            if let Ok(temp) = qmc5883l.temp() {
                critical_section::with(|cs| {
                    println!("mag:{:?}temp:{}", mag, temp);
                    COMPASS_STATE.borrow(cs).set(CompassState { temp, mag });
                });
            }
        }
        ticker.next().await;
    }
}
