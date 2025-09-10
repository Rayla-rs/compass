#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::cell::Cell;

use compass::display::display_task;
use compass::gps::{gps_task, NavPvtState};
use compass::hmc5883i::{magnetometer_task, MagnetometerState};
use compass::led_ring::led_ring_task;
use critical_section::Mutex;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{self, InputConfig, RtcPinWithResistors};
use esp_hal::peripherals::*;
use esp_hal::rtc_cntl::sleep::{Ext1WakeupSource, WakeupLevel};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::timer::systimer::SystemTimer;
use log::info;

static NAV_PVT_STATE: Mutex<Cell<NavPvtState>> = Mutex::new(Cell::new(NavPvtState::new()));
static MAGNETOMETER_STATE: Mutex<Cell<MagnetometerState>> =
    Mutex::new(Cell::new(MagnetometerState::new()));

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
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    // Spawn Tasks
    spawner.must_spawn(button_deep_sleep_task(peripherals.GPIO4, peripherals.LPWR));
    spawner.must_spawn(gps_task(peripherals.UART0, &NAV_PVT_STATE));
    spawner.must_spawn(magnetometer_task(peripherals.I2C0, &MAGNETOMETER_STATE));
    spawner.must_spawn(display_task(
        peripherals.SPI2,
        peripherals.GPIO19,
        peripherals.GPIO18,
        peripherals.GPIO20,
        peripherals.GPIO0,
        peripherals.GPIO1,
        peripherals.GPIO21,
        &NAV_PVT_STATE,
        &MAGNETOMETER_STATE,
    ));
    spawner.must_spawn(led_ring_task(
        peripherals.RMT,
        peripherals.GPIO2,
        &NAV_PVT_STATE,
        &MAGNETOMETER_STATE,
    ));

    loop {
        info!("Hello World");
        Timer::after(Duration::from_secs(1)).await;
    }
}

/// Task that awaits button press to shutdown the esp
#[embassy_executor::task]
async fn button_deep_sleep_task(mut pin: GPIO4<'static>, lpwr: LPWR<'static>) -> ! {
    let mut rtc = Rtc::new(lpwr);

    let mut button = gpio::Input::new(
        pin.reborrow(),
        InputConfig::default().with_pull(gpio::Pull::Down),
    );
    button.wait_for_falling_edge().await;
    core::mem::drop(button);

    let wakeup_pins: &mut [(&mut dyn RtcPinWithResistors, WakeupLevel)] =
        &mut [(&mut pin, WakeupLevel::Low)];
    let ext1 = Ext1WakeupSource::new(wakeup_pins);
    rtc.sleep_deep(&[&ext1]);
}
