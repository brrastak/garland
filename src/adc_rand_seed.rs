
pub use nb::{block, Error, Result};
pub use embedded_hal::adc::{Channel, OneShot};


/// Read the LSB from ADC 8 times and make a number from them
pub fn adc_seed<T: OneShot<U, u32, V>, V: Channel<U>, U>(adc: &mut T, mut pin: &mut V) -> Result<u8, T::Error> {

    let mut res = 0u8;
    for _ in 0..8 {

        res <<= 1;
        let value = adc.read(&mut pin)?;
        if (value & 0x01) != 0 {
            res += 1;
        }
    }

    Ok(res)
}


#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::adc::{Mock, Transaction, MockChan0};
    use pretty_assertions::{assert_eq, assert_ne};
    

    #[test]
    fn get_1111_0000() {

        let expectations = [
            Transaction::read(0, 0x0000_0001),
            Transaction::read(0, 0x0000_00FF),
            Transaction::read(0, 0x0000_FFFF),
            Transaction::read(0, 0xFFFF_FFFF),
            Transaction::read(0, 0x0000_0000),
            Transaction::read(0, 0x0000_00FE),
            Transaction::read(0, 0x0000_FFFE),
            Transaction::read(0, 0xFFFF_FFFE),
        ];
        let mut adc = Mock::new(&expectations);

        assert_eq!(0b1111_0000, adc_seed(&mut adc, &mut MockChan0 {}).unwrap());

        adc.done();
    }
}
