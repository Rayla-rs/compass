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

pub struct Display {
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
    pub async fn new(
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

    fn draw(&mut self) {
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        let style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(BinaryColor::On)
            .build();

        Text::with_baseline(
            "Hello World",
            Point::default(),
            text_style,
            embedded_graphics::text::Baseline::Top,
        )
        .draw(&mut self.display_driver)
        .unwrap();
    }
}
