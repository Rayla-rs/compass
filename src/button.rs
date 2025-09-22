use async_button::{ButtonConfig, ButtonEvent};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use esp_hal::{gpio::InputConfig, peripherals::*};

pub const CHANNEL_SIZE: usize = 8;
static CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, CHANNEL_SIZE> = Channel::new();

#[embassy_executor::task]
pub async fn button_task(button_pin: GPIO2<'static>) -> ! {
    let mut button = async_button::Button::new(
        esp_hal::gpio::Input::new(button_pin, InputConfig::default()),
        ButtonConfig::default(),
    );

    loop {
        CHANNEL.send(button.update().await).await;
    }
}

pub fn try_receive() -> Result<ButtonEvent, embassy_sync::channel::TryReceiveError> {
    CHANNEL.try_receive()
}
