#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use chrono::{DateTime, Utc};
use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::uart::{self, RxError, TxError, Uart};
use esp_hal::Async;
use ublox::{
    CfgPrtUartBuilder, FixedLinearBuffer, GnssFixType, InProtoMask, OutProtoMask, PacketRef,
    Parser, Position, UartMode, Velocity,
};

mod device;

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

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());

    //GPS
    let peripherals = esp_hal::init(config);

    let config = uart::Config::default(); // set baudrate here
    let mut uart = Uart::new(peripherals.UART0, config).unwrap(); // setup more

    // let mut nmea = Nmea::default();
    let mut buf = [0u8; 128];

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    loop {}
}
