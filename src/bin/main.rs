#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use blinksy::color::{ColorCorrection, Hsv, HsvHueRainbow};
use blinksy::drivers::ws2812::Ws2812Led;
use blinksy::layout::Layout1d;
use blinksy_esp::rmt::ClocklessRmtDriver;
use embassy_executor::Spawner;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::Drawable;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{self, Level, OutputConfig};
use esp_hal::i2c::master::Config;
use esp_hal::peripherals::GPIO2;
use esp_hal::rmt::{ConstChannelAccess, Rmt, Tx};
use esp_hal::spi::master::Spi;
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::uart::{self, Uart};
use ssd1306::prelude::SPIInterface;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306, Ssd1306Async};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 0.4.0
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz); // change to max if can

    let peripherals = esp_hal::init(config);

    let config = uart::Config::default().with_baudrate(115200);
    let uart_port = Uart::new(peripherals.UART0, config).unwrap().into_async();

    let config = Config::default();
    // let i2c_port = I2c::new(peripherals.I2C0, config).unwrap().into_async();
    // peripherals.LP_I2C0;
    // use lp for compass and normal for screen

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    // let device = device::Device::new(uart_port, i2c_port).await.unwrap();
    let led_ring = LEDRing::new(peripherals.RMT, peripherals.GPIO2);

    // Spi Lcd
    let mut rst_pin = gpio::Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default());
    let cs_pin = gpio::Output::new(peripherals.GPIO24, Level::Low, OutputConfig::default());
    let dc_pin = gpio::Output::new(peripherals.GPIO23, Level::Low, OutputConfig::default());

    let spi = Spi::new(
        peripherals.SPI2,
        esp_hal::spi::master::Config::default().with_frequency(Rate::from_mhz(80)),
    )
    .unwrap()
    .with_sck(peripherals.GPIO19)
    .with_mosi(peripherals.GPIO18)
    .with_miso(peripherals.GPIO20)
    .into_async();
    let spi = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs_pin).unwrap();

    let dev = SPIInterface::new(spi, dc_pin);

    let mut display = Ssd1306Async::new(
        dev,
        DisplaySize128x64,
        ssd1306::prelude::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    display
        .reset(&mut rst_pin, &mut embassy_time::Delay {})
        .await
        .unwrap();
    let raw: ImageRaw<BinaryColor> = ImageRaw::new(&[2], 64);
    let im = Image::new(&raw, Point::default());
    im.draw(&mut display);
    // Drawable::draw(&self, &mut display);

    // I2C OLED
    // let i2c = I2c::new(
    //     peripherals.I2C0,
    //     Config::default().with_frequency(Rate::from_mhz(80)),
    // )
    // .unwrap()
    // .with_sda(peripherals.GPIO0)
    // .with_scl(peripherals.GPIO1);
    // let interface = I2CDisplayInterface::new(i2c.into_async());
    // let mut display_i2c = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
    //     .into_buffered_graphics_mode();
    // display_i2c.init().await.unwrap();

    // display.reset(&mut rst_pin, );
    // display.init(&mut timer).unwrap();

    loop {}
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

        // let mut control = ControlBuilder::new_1d()
        //     .with_layout::<RingLayout>()
        //     .with_pattern::<Rainbow>(RainbowParams::default())
        //     .with_driver(driver)
        //     .build();

        //

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

//(radians /pi*8)%16
