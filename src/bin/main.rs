#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use compass::app::App;
use compass::display::Display;
use compass::gps::Gps;
use compass::hmc5883i::Hmc5883I;
use compass::led_ring::LEDRing;
use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{self, InputConfig};
use esp_hal::peripherals::*;
use esp_hal::rtc_cntl::sleep::GpioWakeupSource;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::timer::systimer::SystemTimer;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz); // change to max if can
    let peripherals = esp_hal::init(config);

    spawner.must_spawn(button_deep_sleep_task(peripherals.GPIO4, peripherals.LPWR));

    let app = App::new(
        Gps::new(peripherals.UART0).await.unwrap(),
        Hmc5883I::new(peripherals.I2C0),
        Display::new(
            peripherals.SPI2,
            peripherals.GPIO19,
            peripherals.GPIO18,
            peripherals.GPIO20,
            peripherals.GPIO0,
            peripherals.GPIO1,
            peripherals.GPIO21,
        )
        .await,
        LEDRing::new(peripherals.RMT, peripherals.GPIO2),
    );

    // TODO -> battery monitor
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    app.run().await
}

// struct Bat<T: AnalogPin> {
//     pin: T,
// }

//(radians /pi*8)%16
// Sleeper

/// Task that awaits button press to shutdown the esp
#[embassy_executor::task]
async fn button_deep_sleep_task(pin: GPIO4<'static>, lpwr: LPWR<'static>) -> ! {
    let mut rtc = Rtc::new(lpwr);

    let mut button = gpio::Input::new(pin, InputConfig::default().with_pull(gpio::Pull::Down));
    button.wait_for_falling_edge().await;

    rtc.sleep_deep(&[&GpioWakeupSource::new()])
}
