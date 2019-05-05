#![no_std]

extern crate embedded_hal as hal;

#[cfg(feature = "graphics")]
extern crate embedded_graphics;

use hal::digital::OutputPin;
use hal::blocking::delay::DelayMs;

use core::iter::IntoIterator;

pub mod spi;
pub mod par8;

const WIDTH: usize = 240;
const HEIGHT: usize = 320;

/// The default orientation is Portrait
pub enum Orientation {
    Portrait,
    PortraitFlipped,
    Landscape,
    LandscapeFlipped,
}

pub trait Interface {
    fn write_iter<I: IntoIterator<Item = u16>>(&mut self, data: I) -> Result<(), ()>;
    fn write_raw(&mut self, data: &[u8]) -> Result<(), ()>;
    fn command(&mut self, cmd: Command, args: &[u8]) -> Result<(), ()>;
}

/// There are two method for drawing to the screen:
/// [draw_raw](struct.Ili9341.html#method.draw_raw) and
/// [draw_iter](struct.Ili9341.html#method.draw_iter).
///
/// In both cases the expected pixel format is rgb565.
///
/// The hardware makes it efficient to draw rectangles on the screen.
///
/// What happens is the following:
///
/// - A drawing window is prepared (with the 2 opposite corner coordinates)
/// - The starting point for drawint is the top left corner of this window
/// - Every pair of bytes received is intepreted as a pixel value in rgb565
/// - As soon as a pixel is received, an internal counter is incremented,
///   and the next word will fill the next pixel (the adjacent on the right, or
///   the first of the next row if the row ended)
pub struct Ili9341<INTERFACE, RESET> {
    interface: INTERFACE,
    reset: RESET,
    width: usize,
    height: usize,
}

impl<INTERFACE, RESET> Ili9341<INTERFACE, RESET>
where
    INTERFACE: Interface,
    RESET: OutputPin,
{
    pub fn new<DELAY: DelayMs<u16>>(
        interface: INTERFACE,
        reset: RESET,
        delay: &mut DELAY,
    ) -> Result<Self, ()> {
        let mut ili9341 = Ili9341 {
            interface,
            reset,
            width: WIDTH,
            height: HEIGHT,
        };

        ili9341.hard_reset(delay);
        ili9341.interface.command(Command::SoftwareReset, &[])?;
        delay.delay_ms(200);

        ili9341
            .interface
            .command(Command::PowerControlA, &[0x39, 0x2c, 0x00, 0x34, 0x02])?;
        ili9341
            .interface
            .command(Command::PowerControlB, &[0x00, 0xc1, 0x30])?;
        ili9341
            .interface
            .command(Command::DriverTimingControlA, &[0x85, 0x00, 0x78])?;
        ili9341
            .interface
            .command(Command::DriverTimingControlB, &[0x00, 0x00])?;
        ili9341
            .interface
            .command(Command::PowerOnSequenceControl, &[0x64, 0x03, 0x12, 0x81])?;
        ili9341
            .interface
            .command(Command::PumpRatioControl, &[0x20])?;
        ili9341.interface.command(Command::PowerControl1, &[0x23])?;
        ili9341.interface.command(Command::PowerControl2, &[0x10])?;
        ili9341
            .interface
            .command(Command::VCOMControl1, &[0x3e, 0x28])?;
        ili9341.interface.command(Command::VCOMControl2, &[0x86])?;
        ili9341
            .interface
            .command(Command::MemoryAccessControl, &[0x48])?;
        ili9341
            .interface
            .command(Command::PixelFormatSet, &[0x55])?;
        ili9341
            .interface
            .command(Command::FrameControlNormal, &[0x00, 0x18])?;
        ili9341
            .interface
            .command(Command::DisplayFunctionControl, &[0x08, 0x82, 0x27])?;
        ili9341.interface.command(Command::Enable3G, &[0x00])?;
        ili9341.interface.command(Command::GammaSet, &[0x01])?;
        ili9341.interface.command(
            Command::PositiveGammaCorrection,
            &[
                0x0f, 0x31, 0x2b, 0x0c, 0x0e, 0x08, 0x4e, 0xf1, 0x37, 0x07, 0x10, 0x03, 0x0e, 0x09,
                0x00,
            ],
        )?;
        ili9341.interface.command(
            Command::NegativeGammaCorrection,
            &[
                0x00, 0x0e, 0x14, 0x03, 0x11, 0x07, 0x31, 0xc1, 0x48, 0x08, 0x0f, 0x0c, 0x31, 0x36,
                0x0f,
            ],
        )?;
        ili9341.interface.command(Command::SleepOut, &[])?;
        delay.delay_ms(120);
        ili9341.interface.command(Command::DisplayOn, &[])?;

        Ok(ili9341)
    }

    fn hard_reset<DELAY: DelayMs<u16>>(&mut self, delay: &mut DELAY) {
        // set high if previously low
        self.reset.set_high();
        delay.delay_ms(200);
        // set low for reset
        self.reset.set_low();
        delay.delay_ms(200);
        // set high for normal operation
        self.reset.set_high();
        delay.delay_ms(200);
    }

    fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(),()> {
        self.interface.command(
            Command::ColumnAddressSet,
            &[
                (x0 >> 8) as u8,
                (x0 & 0xff) as u8,
                (x1 >> 8) as u8,
                (x1 & 0xff) as u8,
            ],
        )?;
        self.interface.command(
            Command::PageAddressSet,
            &[
                (y0 >> 8) as u8,
                (y0 & 0xff) as u8,
                (y1 >> 8) as u8,
                (y1 & 0xff) as u8,
            ],
        )?;
        Ok(())
    }
    /// Draw a rectangle on the screen, represented by top-left corner (x0, y0)
    /// and bottom-right corner (x1, y1).
    ///
    /// The border is included.
    ///
    /// This method accepts an iterator of rgb565 pixel values.
    ///
    /// The iterator is useful to avoid wasting memory by holding a buffer for
    /// the whole screen when it is not necessary.
    pub fn draw_iter<I: IntoIterator<Item = u16>>(
        &mut self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        data: I,
    ) -> Result<(), ()> {
        self.set_window(x0, y0, x1, y1)?;
        self.interface.write_iter(data)
    }
    /// Draw a rectangle on the screen, represented by top-left corner (x0, y0)
    /// and bottom-right corner (x1, y1).
    ///
    /// The border is included.
    ///
    /// This method accepts a raw buffer of bytes that will be copied to the screen
    /// video memory.
    ///
    /// The expected format is rgb565, and the two bytes for a pixel
    /// are in big endian order.
    pub fn draw_raw(
        &mut self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        data: &[u8],
    ) -> Result<(), ()> {
        self.set_window(x0, y0, x1, y1)?;
        self.interface.write_raw(data)
    }
    /// Change the orientation of the screen
    pub fn set_orientation(&mut self, mode: Orientation) -> Result<(), ()> {
        match mode {
            Orientation::Portrait => {
                self.width = WIDTH;
                self.height = HEIGHT;
                self.interface
                    .command(Command::MemoryAccessControl, &[0x40 | 0x08])
            }
            Orientation::Landscape => {
                self.width = HEIGHT;
                self.height = WIDTH;
                self.interface
                    .command(Command::MemoryAccessControl, &[0x20 | 0x08])
            }
            Orientation::PortraitFlipped => {
                self.width = WIDTH;
                self.height = HEIGHT;
                self.interface
                    .command(Command::MemoryAccessControl, &[0x80 | 0x08])
            }
            Orientation::LandscapeFlipped => {
                self.width = HEIGHT;
                self.height = WIDTH;
                self.interface
                    .command(Command::MemoryAccessControl, &[0x40 | 0x80 | 0x20 | 0x08])
            }
        }
    }
    /// Get the current screen width. It can change based on the current orientation
    pub fn width(&self) -> usize {
        self.width
    }
    /// Get the current screen heighth. It can change based on the current orientation
    pub fn height(&self) -> usize {
        self.height
    }
}

#[cfg(feature = "graphics")]
use embedded_graphics::drawable;
#[cfg(feature = "graphics")]
use embedded_graphics::{drawable::Pixel, pixelcolor::PixelColorU16, Drawing};

#[cfg(feature = "graphics")]
impl<INTERFACE, RESET> Drawing<PixelColorU16> for Ili9341<INTERFACE, RESET>
where
    INTERFACE: Interface,
    RESET: OutputPin,
{
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = drawable::Pixel<PixelColorU16>>,
    {
        for Pixel(pos, color) in item_pixels {
            self.draw_raw(
                pos.0 as u16,
                pos.1 as u16,
                pos.0 as u16,
                pos.1 as u16,
                if color == PixelColorU16(0) {
                    &[0xff, 0xff]
                } else {
                    &[0, 0]
                },
            )
            .expect("Failed to communicate with device");
        }
    }
}

#[derive(Clone, Copy)]
pub enum Command {
    SoftwareReset = 0x01,
    PowerControlA = 0xcb,
    PowerControlB = 0xcf,
    DriverTimingControlA = 0xe8,
    DriverTimingControlB = 0xea,
    PowerOnSequenceControl = 0xed,
    PumpRatioControl = 0xf7,
    PowerControl1 = 0xc0,
    PowerControl2 = 0xc1,
    VCOMControl1 = 0xc5,
    VCOMControl2 = 0xc7,
    MemoryAccessControl = 0x36,
    PixelFormatSet = 0x3a,
    FrameControlNormal = 0xb1,
    DisplayFunctionControl = 0xb6,
    Enable3G = 0xf2,
    GammaSet = 0x26,
    PositiveGammaCorrection = 0xe0,
    NegativeGammaCorrection = 0xe1,
    SleepOut = 0x11,
    DisplayOn = 0x29,
    ColumnAddressSet = 0x2a,
    PageAddressSet = 0x2b,
    MemoryWrite = 0x2c,
}
