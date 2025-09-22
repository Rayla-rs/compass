use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::{gpio, peripherals::*, Async};
use esp_hal::{spi::master::Spi, time::Rate};
use pcd8544_hal::{Pcd8544, Pcd8544Spi};

use crate::user_interface::sprites::Frame;

pub enum DrawCommand {
    Char(u8),
    Str(&'static str),
    Frame(Frame),
    SetPos(u8, u8),
    Clear,
}

pub struct Display {
    display_driver: Pcd8544Spi<Spi<'static, Async>, Output<'static>, Output<'static>>,
}

impl Display {
    pub async fn new(
        spi: SPI2<'static>,
        sck: GPIO19<'static>,
        mosi: GPIO18<'static>,
        miso: GPIO20<'static>,

        rst: GPIO0<'static>,
        cs: GPIO1<'static>,
        dc: GPIO21<'static>,
    ) -> Self {
        let mut rst_pin = gpio::Output::new(rst, Level::Low, OutputConfig::default());
        let cs_pin = gpio::Output::new(cs, Level::Low, OutputConfig::default());
        let dc_pin = gpio::Output::new(dc, Level::Low, OutputConfig::default());

        // Setup driver

        let spi = Spi::new(
            spi,
            esp_hal::spi::master::Config::default().with_frequency(Rate::from_mhz(40)),
        )
        .unwrap()
        .with_sck(sck)
        .with_mosi(mosi)
        .with_miso(miso)
        .into_async();

        let mut delay = Delay::new();
        let display_driver = Pcd8544Spi::new(spi, dc_pin, cs_pin, Some(&mut rst_pin), &mut delay);

        Self { display_driver }
    }

    pub fn execute(&mut self, command: &'static DrawCommand) {
        match command {
            DrawCommand::Char(c) => self.display_driver.print_char(*c),
            DrawCommand::Str(s) => self.display_driver.print(s),
            DrawCommand::Frame(buffer) => self.display_driver.draw_buffer(buffer),
            DrawCommand::SetPos(x, y) => self.display_driver.set_position(*x, *y),
            DrawCommand::Clear => self.display_driver.clear(),
        }
    }
}
