#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::cell::Cell;

use compass::gps::{gps_task, NavPvtState};
use compass::hmc5883i::{magnetometer_task, MagnetometerState};
use compass::led_ring::led_ring_task;
use critical_section::Mutex;
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
    // Board setup
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz); // change to max if can
    let peripherals = esp_hal::init(config);

    // Create States
    let nav_pvt_state = Mutex::new(Cell::new(NavPvtState::new()));
    let magnetometer_state = Mutex::new(Cell::new(MagnetometerState::default()));

    // Spawn Tasks
    spawner.must_spawn(button_deep_sleep_task(peripherals.GPIO4, peripherals.LPWR));
    spawner.must_spawn(gps_task(peripherals.UART0, nav_pvt_state));
    spawner.must_spawn(magnetometer_task(peripherals.I2C0, magnetometer_state));
    spawner.must_spawn(display_task(
        spi,
        sck,
        mosi,
        miso,
        rst,
        cs,
        dc,
        nav_pvt_state,
        magnetometer_state,
    ));
    spawner.spawn(led_ring_task(
        peripherals.RMT,
        peripherals.GPIO2,
        nav_pvt_state,
        magnetometer_state,
    ));

    // TODO -> battery monitor
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    app.run().await
}

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
