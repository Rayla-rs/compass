use core::cell::Cell;

use critical_section::Mutex;

use crate::{generated, landmark::Landmark};

pub static UI: Mutex<Cell<UserInterface>> =
    Mutex::new(Cell::new(UserInterface { landmark_index: 0 }));

enum Menu {
    Boot,
    Time,
    Compass,
}

//impl into bitmap
// buttons
// we have 14 chars x 6
//
//
// 12345678901234
//   dd:mm:yyyy
//    hh:mm:ss
//

pub struct UserInterface {
    landmark_index: usize,
}

impl UserInterface {
    pub fn next_landmark(&mut self) {
        self.landmark_index = (self.landmark_index + 1) % generated::LANDMARKS.len();
    }

    pub fn previouse_landmark(&mut self) {
        self.landmark_index = match self.landmark_index {
            0 => generated::LANDMARKS.len() - 1,
            _ => self.landmark_index - 1,
        }
    }

    pub fn current_landmark(&mut self) -> &'static Landmark {
        &generated::LANDMARKS[self.landmark_index]
    }

    // produce buffer for display
}

mod screen {
    pub const WIDTH: usize = 84;
    pub const HEIGHT: usize = 48;
    const BUF_SIZE: usize = WIDTH * HEIGHT / 8;
    pub struct ScreenBuffer {
        buf: [u8; BUF_SIZE],
    }

    impl Default for ScreenBuffer {
        fn default() -> Self {
            Self {
                buf: [0u8; BUF_SIZE],
            }
        }
    }

    impl ScreenBuffer {
        pub fn set_pixel(&mut self, x: usize, y: usize, value: bool) {
            assert!(x < WIDTH);
            assert!(y < HEIGHT);

            if value {
                self.buf[x + (y / 8) * WIDTH] |= 1 << (y % 8);
            } else {
                self.buf[x + (y / 8) * WIDTH] &= !(1 << (y % 8))
            }
        }
    }
}
