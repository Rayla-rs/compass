use crate::{display::Display, user_interface::UserInterface};
use embassy_time::{Duration, Ticker};

pub struct App {
    display: Display,
    ui: UserInterface,
}

impl App {
    pub fn new(display: Display) -> Self {
        Self {
            display,
            ui: UserInterface::default(),
        }
    }

    pub async fn run(mut self) -> ! {
        let mut ticker = Ticker::every(Duration::from_millis(200));

        loop {
            // Receive button events
            if let Ok(event) = crate::button::try_receive() {
                self.ui.process_input(event);
            }

            // Update ui and display
            for command in self.ui.process() {
                self.display.execute(command);
            }

            // Sleep
            ticker.next().await;
        }
    }
}
