#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use compass::device;
use compass::display::Display;
use compass::led_ring::LEDRing;
use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz); // change to max if can
    let peripherals = esp_hal::init(config);

    // GPS
    let _device = device::Device::new(peripherals.UART0, peripherals.I2C0)
        .await
        .unwrap();

    // Led Ring
    let _led_ring = LEDRing::new(peripherals.RMT, peripherals.GPIO2);

    // Display
    let _display = Display::new(
        peripherals.SPI2,
        peripherals.GPIO19,
        peripherals.GPIO18,
        peripherals.GPIO20,
        peripherals.GPIO0,
        peripherals.GPIO1,
        peripherals.GPIO21,
    );

    // TODO -> battery monitor

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    loop {}
}

//(radians /pi*8)%16
