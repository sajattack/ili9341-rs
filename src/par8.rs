use hal::digital::OutputPin;

use Interface;
use Command;

pub struct Par8<DATA0, DATA1, DATA2, DATA3, DATA4, DATA5, DATA6, DATA7, DC, WR> {
    data0: DATA0,
    data1: DATA1,
    data2: DATA2,
    data3: DATA3,
    data4: DATA4,
    data5: DATA5,
    data6: DATA6,
    data7: DATA7,
    dc: DC,
    wr: WR,
}

impl<DATA0, DATA1, DATA2, DATA3, DATA4, DATA5, DATA6, DATA7, DC, WR> Par8<DATA0, DATA1, DATA2, DATA3, DATA4, DATA5, DATA6, DATA7, DC, WR>
where 
    DATA0: OutputPin,
    DATA1: OutputPin,
    DATA2: OutputPin,
    DATA3: OutputPin,
    DATA4: OutputPin,
    DATA5: OutputPin,
    DATA6: OutputPin,
    DATA7: OutputPin,
    DC: OutputPin,
    WR: OutputPin
{
    pub fn new(
        data0: DATA0,
        data1: DATA1,
        data2: DATA2,
        data3: DATA3,
        data4: DATA4,
        data5: DATA5,
        data6: DATA6,
        data7: DATA7,
        dc: DC,
        wr: WR
    ) -> Self {
        Self {
            data0,
            data1,
            data2,
            data3,
            data4,
            data5,
            data6,
            data7,
            dc,
            wr
        }
    }
    fn write_bytes(&mut self, data: &[u8]) {
        for byte in data { 
            self.wr.set_low();
            if byte & (1 >> 0) == 1 {
                self.data0.set_high();
            } else {
                self.data0.set_low();
            }
            if byte & (1 >> 1) == 1 {
                self.data1.set_high();
            } else {
                self.data1.set_low();
            }
            if byte & (1 >> 2) == 1 {
                self.data2.set_high();
            } else {
                self.data2.set_low();
            }
            if byte & (1 >> 3) == 1 {
                self.data3.set_high();
            } else {
                self.data3.set_low();
            }
            if byte & (1 >> 4) == 1 {
                self.data4.set_high();
            } else {
                self.data4.set_low();
            }
            if byte & (1 >> 5) == 1 {
                self.data5.set_high();
            } else {
                self.data5.set_low();
            }
            if byte & (1 >> 6) == 1 {
                self.data6.set_high();
            } else {
                self.data6.set_low();
            }
            if byte & (1 >> 7) == 1 {
                self.data7.set_high();
            } else {
                self.data7.set_low();
            }
            self.wr.set_high();
        }
    }
}

impl <DATA0, DATA1, DATA2, DATA3, DATA4, DATA5, DATA6, DATA7, DC, WR> Interface for Par8<DATA0, DATA1, DATA2, DATA3, DATA4, DATA5, DATA6, DATA7, DC, WR>
where 
    DATA0: OutputPin,
    DATA1: OutputPin,
    DATA2: OutputPin,
    DATA3: OutputPin,
    DATA4: OutputPin,
    DATA5: OutputPin,
    DATA6: OutputPin,
    DATA7: OutputPin,
    DC: OutputPin,
    WR: OutputPin
{

    fn write_iter<I: IntoIterator<Item = u16>>(&mut self, data: I) -> Result<(), ()> {
        self.dc.set_low();
        self.write_bytes(&[Command::MemoryWrite as u8]);
        self.dc.set_high();
        for d in data.into_iter() {
            self.write_bytes(&[(d >> 8) as u8, (d & 0xff) as u8]);
        }
        Ok(())
    }
    
    fn write_raw(&mut self, data: &[u8]) -> Result<(), ()> {
        self.dc.set_low();
        self.write_bytes(&[Command::MemoryWrite as u8]);
        self.dc.set_high();
        self.write_bytes(data);
        Ok(())
    }

    fn command(&mut self, cmd: Command, args: &[u8]) -> Result<(), ()> {
        self.dc.set_low();
        self.write_bytes(&[cmd as u8]);
        self.dc.set_high();
        self.write_bytes(args);
        Ok(())
    }
}
