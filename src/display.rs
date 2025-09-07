use arrform::{arrform, ArrForm};
use core::cell::Cell;
use critical_section::Mutex;
use embassy_time::{Duration, Ticker};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::text::Text;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::{gpio, peripherals::*, Async};
use esp_hal::{spi::master::Spi, time::Rate};
use ssd1306::mode::BufferedGraphicsModeAsync;
use ssd1306::prelude::*;
use ssd1306::Ssd1306Async;

use crate::gps::NavPvtState;
use crate::hmc5883i::MagnetometerState;

#[embassy_executor::task]
pub async fn display_task(
    spi: SPI2<'static>,
    sck: GPIO19<'static>,
    mosi: GPIO18<'static>,
    miso: GPIO20<'static>,

    rst: GPIO0<'static>,
    cs: GPIO1<'static>,
    dc: GPIO21<'static>,

    nav_pvt_state: &'static Mutex<Cell<NavPvtState>>,
    magnetometer_state: &'static Mutex<Cell<MagnetometerState>>,
) -> ! {
    let mut display = Display::new(spi, sck, mosi, miso, rst, cs, dc).await;
    let mut ticker = Ticker::every(Duration::from_millis(50));

    loop {
        let argument = critical_section::with(|cs| {
            let nav_pvt_state = nav_pvt_state.borrow(cs).get();
            let _magnetometer_state = magnetometer_state.borrow(cs).get();

            let header = arrform!(
                64,
                "T:{}{}{} Sats:{}\nLat:{}\nLon:{}",
                nav_pvt_state.hour,
                nav_pvt_state.min,
                nav_pvt_state.sec,
                nav_pvt_state.satellites_used,
                nav_pvt_state.lat,
                nav_pvt_state.lon
            );

            DisplayArgument { header }
        });

        display.process(argument);
        ticker.next().await;
    }
}

struct DisplayArgument {
    header: ArrForm<64>,
}

struct Display {
    display_driver: Ssd1306Async<
        SPIInterface<
            ExclusiveDevice<Spi<'static, Async>, Output<'static>, NoDelay>,
            Output<'static>,
        >,
        DisplaySize128x64,
        BufferedGraphicsModeAsync<DisplaySize128x64>,
    >,
}

impl Display {
    async fn new(
        spi: SPI2<'static>,
        sck: GPIO19<'static>,
        mosi: GPIO18<'static>,
        miso: GPIO20<'static>,

        rst: GPIO0<'static>,
        cs: GPIO1<'static>,
        dc: GPIO21<'static>,
    ) -> Self {
        // Spi Lcd
        let mut rst_pin = gpio::Output::new(rst, Level::Low, OutputConfig::default());
        let cs_pin = gpio::Output::new(cs, Level::Low, OutputConfig::default());
        let dc_pin = gpio::Output::new(dc, Level::Low, OutputConfig::default());

        // Setup driver

        let spi = Spi::new(
            spi,
            esp_hal::spi::master::Config::default().with_frequency(Rate::from_mhz(80)),
        )
        .unwrap()
        .with_sck(sck)
        .with_mosi(mosi)
        .with_miso(miso)
        .into_async();

        // Setup device
        let spi = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs_pin).unwrap();

        // Setup Interface
        let interface = SPIInterface::new(spi, dc_pin);

        let mut display_driver = Ssd1306Async::new(
            interface,
            DisplaySize128x64,
            ssd1306::prelude::DisplayRotation::Rotate0,
        )
        .into_buffered_graphics_mode();
        display_driver
            .reset(&mut rst_pin, &mut embassy_time::Delay {})
            .await
            .unwrap();
        display_driver.init().await.unwrap();

        Self { display_driver }
    }

    fn process(&mut self, arg: DisplayArgument) {
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        let _style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(BinaryColor::On)
            .build();

        Text::with_baseline(
            arg.header.as_str(),
            Point::default(),
            text_style,
            embedded_graphics::text::Baseline::Top,
        )
        .draw(&mut self.display_driver)
        .unwrap();
    }
}
