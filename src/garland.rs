
pub use smart_leds::{SmartLedsWrite, RGB8};


/// Number of WS2812 LEDs in the LED strip
pub const LED_NUMBER: usize = 300;
/// Store all the LED color values for the LED strip
pub type ColorFrame = [RGB8; LED_NUMBER];

/// Make pastel color not so pastel by decreasing two of three RGB components
pub fn no_pastel(color: RGB8) -> RGB8 {

    let mut res = color;

    if color.r == max(color.r, color.g, color.b) {
        res.g /= 3;
        res.b /= 3;
    } else if color.g == max(color.r, color.g, color.b) {
        res.r /= 3;
        res.b /= 3;
    } else {
        res.r /= 3 + 1;
        res.g /= 3 + 1;
    }

    res
}

fn max(one: u8, two: u8, three: u8) -> u8 {
    max2(one, max2(two, three))
}

fn max2(one: u8, two: u8) -> u8 {
    if one > two {
        one
    } else {
        two
    }
}
