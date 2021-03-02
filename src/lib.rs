#![cfg_attr(not(test), no_std)]

use crc_any::CRCu8;
use embedded_hal::blocking::i2c;

/// SCD30 I2C address
const ADDRESS: u8 = 0x61;

/// A SCD30 sensor on the I2C bus `I`
pub struct Scd30<I>(I)
where
    I: i2c::Read + i2c::Write;

/// A driver error
#[derive(Debug, PartialEq)]
pub enum Error<E> {
    /// I2C bus error
    I2c(E),
    /// CRC validation failed
    InvalidCrc,
}

impl<E, I> Scd30<I>
where
    I: i2c::Read<Error = E> + i2c::Write<Error = E>,
{
    /// Initializes the SCD30 driver.
    /// This consumes the I2C bus `I`
    pub fn init(i2c: I) -> Self {
        Scd30(i2c)
    }

    /// Returns the firmware version reported by the SCD30 sensor
    pub fn get_firmware_version(&mut self) -> Result<[u8; 2], Error<E>> {
        let command: [u8; 2] = [0xd1, 0x00];
        let mut rd_buffer = [0u8; 3];

        self.0.write(ADDRESS, &command).map_err(Error::I2c)?;
        self.0.read(ADDRESS, &mut rd_buffer).map_err(Error::I2c)?;

        let major = rd_buffer[0];
        let minor = rd_buffer[1];
        let crc = rd_buffer[2];

        if compute_crc(&rd_buffer[..2]) == crc {
            Ok([major, minor])
        } else {
            Err(Error::InvalidCrc)
        }
    }

    /// Destroys this driver and releases the I2C bus `I`
    pub fn destroy(self) -> I {
        self.0
    }
}

fn compute_crc(bytes: &[u8]) -> u8 {
    let mut crc = CRCu8::create_crc(0x31, 8, 0xff, 0x00, false);
    crc.digest(bytes);
    crc.get_crc()
}

#[cfg(test)]
mod tests {
    use super::{Error, Scd30, ADDRESS};
    use embedded_hal_mock::i2c;

    #[test]
    fn firmware_version() {
        let expectations = vec![
            i2c::Transaction::write(ADDRESS, vec![0xD1, 0x00]),
            i2c::Transaction::read(ADDRESS, vec![0x03, 0x42, 0xF3]),
        ];
        let mock = i2c::Mock::new(&expectations);

        let mut scd30 = Scd30::init(mock);
        let version = scd30.get_firmware_version().unwrap();
        assert_eq!([3, 66], version);

        let mut mock = scd30.destroy();
        mock.done(); // verify expectations
    }

    #[test]
    fn firmware_version_bad_crc() {
        let expectations = vec![
            i2c::Transaction::write(ADDRESS, vec![0xD1, 0x00]),
            // NOTE negated CRC byte in the response!
            i2c::Transaction::read(ADDRESS, vec![0x03, 0x42, !0xF3]),
        ];
        let mock = i2c::Mock::new(&expectations);

        let mut scd30 = Scd30::init(mock);
        let res = scd30.get_firmware_version();
        assert_eq!(Err(Error::InvalidCrc), res);

        scd30.destroy().done(); // verify expectations
    }

    #[test]
    fn crc() {
        // example from the Interface Specification document
        assert_eq!(super::compute_crc(&[0xBE, 0xEF]), 0x92);
    }
}
