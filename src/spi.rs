use hal::spi::{Mode, Phase, Polarity};
use hal::blocking::spi;
use hal::digital::OutputPin;

use Interface;
use Command;

/// SPI mode
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

pub struct Spi<SPI, CS, DC> {
    spi: SPI,
    cs: CS,
    dc: DC,
}

impl<SPI, CS, DC> Spi<SPI, CS, DC>
where
    SPI: spi::Transfer<u8, Error = ()> + spi::Write<u8, Error = ()>,
    CS: OutputPin,
    DC: OutputPin,
{
    pub fn new(
        spi: SPI,
        cs: CS,
        dc: DC,
    ) -> Self {
        let spi = Spi {
            spi,
            cs,
            dc,
        };
        spi
    }
}

impl<SPI,CS,DC> Interface for Spi<SPI, CS, DC>
where
    SPI: spi::Transfer<u8, Error = ()> + spi::Write<u8, Error = ()>,
    CS: OutputPin,
    DC: OutputPin,
{
    fn write_iter<I: IntoIterator<Item = u16>>(&mut self, data: I) -> Result<(), ()> {
        self.cs.set_low();

        self.dc.set_low();
        self.spi
            .write(&[Command::MemoryWrite as u8])
            .map_err(|_|())?;

        self.dc.set_high();
        for d in data.into_iter() {
            self.spi
                .write(&[(d >> 8) as u8, (d & 0xff) as u8])
                .map_err(|_|())?;

        }

        self.cs.set_high();
        Ok(())
    }

    fn write_raw(&mut self, data: &[u8]) -> Result<(), ()> {
        self.cs.set_low();

        self.dc.set_low();
        self.spi
            .write(&[Command::MemoryWrite as u8])
            .map_err(|_|())?;

        self.dc.set_high();
        self.spi.write(data).map_err(|_|())?;
        self.cs.set_high();
        Ok(())
    }

    fn command(&mut self, cmd: Command, args: &[u8]) -> Result<(), ()> {
        self.cs.set_low();

        self.dc.set_low();
        self.spi.write(&[cmd as u8]).map_err(|_|())?;

        self.dc.set_high();
        self.spi.write(args).map_err(|_|())?;

        self.cs.set_high();
        Ok(())
    }
}
