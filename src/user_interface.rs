use arrform::*;
use core::cell::Cell;
use critical_section::Mutex;
use pcd8544_hal::Pcd8544;

use crate::{
    generated,
    gps::NAV_PVT_STATE,
    landmark::Landmark,
    user_interface::{
        screen::ScreenBuffer,
        sprites::{Anim, Frame},
    },
};

mod sprites {
    use embassy_time::Duration;

    pub type Frame = &'static [u8; 504];
    pub const FRAME_DURATION: Duration = Duration::from_millis(250);

    pub struct Anim {
        frames: &'static [Frame],
    }

    impl Anim {
        pub fn next(self) -> Option<Anim> {
            if self.frames.len() > 1 {
                Some(Anim {
                    frames: &self.frames[1..],
                })
            } else {
                None
            }
        }

        /// Returns the current frame. Paniks if Anim was poorly initialized with zero frames
        pub fn current(&self) -> Frame {
            self.frames[0]
        }
    }

    const BLINK1: Frame = include_bytes!("./assets/blink_1.bin");
    const BLINK2: Frame = include_bytes!("./assets/blink_2.bin");
    const BLINK3: Frame = include_bytes!("./assets/blink_3.bin");
    const BLINK4: Frame = include_bytes!("./assets/blink_4.bin");
    pub const BLINK: Anim = Anim {
        frames: &[BLINK1, BLINK2, BLINK3, BLINK4],
    };

    const DUCK1: Frame = include_bytes!("./assets/duck_1.bin");
    const DUCK2: Frame = include_bytes!("./assets/duck_2.bin");
    const DUCK3: Frame = include_bytes!("./assets/duck_3.bin");
    pub const DUCK: Anim = Anim {
        frames: &[DUCK1, DUCK2, DUCK3],
    };

    const IDLE1: Frame = include_bytes!("./assets/idle_1.bin");
    const IDLE2: Frame = include_bytes!("./assets/idle_2.bin");
    const IDLE3: Frame = include_bytes!("./assets/idle_3.bin");
    const IDLE4: Frame = include_bytes!("./assets/idle_4.bin");
    pub const IDLE: Anim = Anim {
        frames: &[IDLE1, IDLE2, IDLE3, IDLE4],
    };

    const BUBBLE1: Frame = include_bytes!("./assets/bubble_1.bin");
    const BUBBLE2: Frame = include_bytes!("./assets/bubble_2.bin");
    const BUBBLE3: Frame = include_bytes!("./assets/bubble_3.bin");
    const BUBBLE4: Frame = include_bytes!("./assets/bubble_4.bin");
    pub const BUBBLE: Anim = Anim {
        frames: &[BUBBLE1, BUBBLE2, BUBBLE3, BUBBLE4],
    };
}

pub static UI: Mutex<Cell<UserInterface>> = Mutex::new(Cell::new(UserInterface {
    landmark_index: 0,
    menu: Menu::Boot,
    anim: None,
}));

enum Menu {
    Boot,
    Neko,
    Time,
    Compass,
}

impl Menu {
    pub fn draw(&self, display: &mut impl Pcd8544) {
        display.clear();
        match self {
            Menu::Boot => {
                display.draw_buffer(include_bytes!("./assets/rust_logo.bin"));
            }
            Menu::Time => {
                critical_section::with(|cs| {
                    let state = NAV_PVT_STATE.borrow(cs).get();
                    display.print(
                        arrform!(
                            64,
                            "  {day:02}:{month:02}:{year:04}  \n   {hour:02}:{min:02}:{sec:02}   ",
                            day = state.day,
                            month = state.month,
                            year = state.year,
                            hour = state.hour,
                            min = state.min,
                            sec = state.sec
                        )
                        .as_str(),
                    );
                });
            }
            _ => {}
        }
    }
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
//
//
// 12345678901234

pub struct UserInterface {
    landmark_index: usize,
    menu: Menu,
    anim: Option<Anim>,
}

// like this for changing menus
// async fn transition() {}
enum UI {
    Menu,
    Transition(Anim),
}

impl UserInterface {
    pub async fn run(self) -> ! {
        // draw boot logo
        // wait
        // start main loop

        loop {}
    }

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
        pub fn clear(&mut self) {
            self.buf = [0u8; BUF_SIZE];
        }
        // TODO
        // line
        // triangle

        fn draw_circle(&mut self, centre: (usize, usize), x: usize, y: usize, value: bool) {
            self.set_pixel(centre.0 + x, centre.1 + y, value);
            self.set_pixel(centre.0 - x, centre.1 + y, value);
            self.set_pixel(centre.0 + x, centre.1 - y, value);
            self.set_pixel(centre.0 - x, centre.1 - y, value);
            self.set_pixel(centre.0 + y, centre.1 + x, value);
            self.set_pixel(centre.0 - y, centre.1 + x, value);
            self.set_pixel(centre.0 + y, centre.1 - x, value);
            self.set_pixel(centre.0 - y, centre.1 - x, value);
        }

        /// Bresenham's algorithm
        pub fn circle(&mut self, centre: (usize, usize), radius: usize, value: bool) {
            let mut x = 0;
            let mut y = radius;
            let mut d = 3 - 2 * radius;
            self.draw_circle(centre, x, y, value);
            while y <= x {
                d += if d > 0 {
                    y -= 1;
                    4 * (x - y) + 10
                } else {
                    4 * x + 6
                };
                x += 1;
                self.draw_circle(centre, x, y, value);
            }
        }

        pub fn line(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, value: bool) {
            let (ix1, iy1, ix2, iy2) = (x1 as isize, y1 as isize, x2 as isize, y2 as isize);
            let m_new = 2 * (iy2 - iy1);
            let mut slope_err_new = m_new - (ix2 - ix1);
            let mut x = x1;
            let mut y = y1;
            while x <= x2 {
                self.set_pixel(x, y, value);
                slope_err_new += m_new;

                if slope_err_new >= 0 {
                    y += 1;
                    slope_err_new -= 2 * (ix2 - ix1);
                }

                x += 1
            }
        }

        pub fn test() {
            // embedded_graphics::framebuffer::
        }
    }
}

pub enum DrawCommand {
    Char(u8),
    Str(&'static str),
    Frame(Frame),
    SetPos(u8, u8),
    Clear,
}
