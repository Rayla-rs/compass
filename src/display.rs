// use embedded_graphics::mono_font::ascii::FONT_6X10;
// use embedded_graphics::mono_font::MonoTextStyleBuilder;
// use embedded_graphics::pixelcolor::{BinaryColor, Gray8};
// use embedded_graphics::prelude::Point;
// use embedded_graphics::prelude::*;
// use embedded_graphics::primitives::PrimitiveStyleBuilder;
// use embedded_graphics::text::{Text, TextStyle};
// use esp_hal::gpio::OutputConfig;
// use esp_hal::{gpio, peripherals::*};
// use esp_hal::{
//     spi::master::{Config, Spi},
//     time::Rate,
//     Blocking,
// };
// use ssd1306::mode::BufferedGraphicsMode;
// use ssd1306::{mode::BasicMode, prelude::*, Ssd1306};

// pub struct Display {
//     driver:
//         Ssd1306<Spi<'static, Blocking>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
// }

// impl Display {
//     pub fn new(
//         // rst: PB2<'static>,
//         spi: SPI2<'static>,
//         sck: GPIO19<'static>,
//         mosi: GPIO18<'static>,
//         miso: GPIO20<'static>,
//     ) -> Self {
//         // let mut rst = gpio::Output::new(sck, gpio::Level::Low, OutputConfig::default());

//         let interface = Spi::new(
//             spi,
//             Config::default()
//                 .with_frequency(Rate::from_khz(100))
//                 .with_mode(esp_hal::spi::Mode::_0),
//         )
//         .unwrap()
//         .with_sck(sck)
//         .with_mosi(mosi)
//         .with_miso(miso);

//         let interface = SPIInterface::new(interface, dc);

//         let mut driver = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
//             .into_buffered_graphics_mode();
//         // esp_hal_embassy::

//         // driver.init().unwrap();

//         Self { driver }
//     }

//     fn draw(&mut self) {
//         let text_style = MonoTextStyleBuilder::new()
//             .font(&FONT_6X10)
//             .text_color(BinaryColor::On)
//             .build();

//         let style = PrimitiveStyleBuilder::new()
//             .stroke_width(1)
//             .stroke_color(BinaryColor::On)
//             .build();

//         let text = Text::with_baseline(
//             "Hello World",
//             Point::default(),
//             text_style,
//             embedded_graphics::text::Baseline::Top,
//         )
//         .draw(&mut self.driver);
//     }
// }

// struct Disp {
//     spi: SPI2<'static>,
// }

// impl DrawTarget for Disp {
//     type Color = Gray8;
//     type Error = core::convert::Infallible;

//     fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
//     where
//         I: IntoIterator<Item = Pixel<Self::Color>>,
//     {
//         for Pixel(coord, color) in pixels.into_iter() {
//             // Check if the pixel coordinates are out of bounds (negative or greater than
//             // (63,63)). `DrawTarget` implementation are required to discard any out of bounds
//             // pixels without returning an error or causing a panic.
//             if let Ok((x @ 0..=63, y @ 0..=63)) = coord.try_into() {
//                 self.set_pixel(x, y, RawU16::from(color).into_inner())?;
//             }
//         }

//         Ok(())
//     }

//     fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
//         // Clamp the rectangle coordinates to the valid range by determining
//         // the intersection of the fill area and the visible display area
//         // by using Rectangle::intersection.
//         let area = area.intersection(&self.bounding_box());

//         // Do not send a draw rectangle command if the intersection size if zero.
//         // The size is checked by using `Rectangle::bottom_right`, which returns `None`
//         // if the size is zero.
//         let bottom_right = if let Some(bottom_right) = area.bottom_right() {
//             bottom_right
//         } else {
//             return Ok(());
//         };

//         self.send_commands(&[
//             // Draw rectangle command
//             0x22,
//             // Top left X coordinate
//             area.top_left.x as u8,
//             // Top left Y coordinate
//             area.top_left.y as u8,
//             // Bottom right X coordinate
//             bottom_right.x as u8,
//             // Bottom right Y coordinate
//             bottom_right.y as u8,
//             // Fill color red channel
//             color.r(),
//             // Fill color green channel
//             color.g(),
//             // Fill color blue channel
//             color.b(),
//         ])
//     }
// }

// impl OriginDimensions for Disp {
//     fn size(&self) -> Size {
//         Size::new(128, 64)
//     }
// }
