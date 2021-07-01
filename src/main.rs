use rppal::i2c::I2c;
use std::error::Error;

const I2C_ADDR_PRIMARY: u16 = 0x0A;
//const I2C_ADDR_ALTERNATIVE: u16 = 0x0B;
const CHIP_ID: u16 = 0xBA11;
//const VERSION: u16 = 1;
const REG_LED_RED: u8 = 0x00;
const REG_LED_GRN: u8 = 0x01;
const REG_LED_BLU: u8 = 0x02;
const REG_LED_WHT: u8 = 0x03;
// const REG_LEFT: u16 = 0x04;
// const REG_RIGHT: u16 = 0x05;
// const REG_UP: u16 = 0x06;
// const REG_DOWN: u16 = 0x07;
// const REG_SWITCH: u16 = 0x08;
// const MSK_CLICKED: u16 = 0x80;
// const MSK_CLICK_STATE_UPDATE: u16 = 0x01;
// const MSK_SWITCH_STATE: u16 = 0b10000000;
// const REG_USER_FLASH: u16 = 0xD0;
// const REG_FLASH_PAGE: u16 = 0xF0;
// const REG_INT: u16 = 0xF9;
// const MSK_INT_TRIGGERED: u16 = 0b00000001;
// const MSK_INT_OUT_EN: u16 = 0b00000010;
 const REG_CHIP_ID_L: u16 = 0xFA;
// const RED_CHIP_ID_H: u16 = 0xFB;
// const REG_VERSION: u16 = 0xFC;
// const REG_I2C_ADDR: u16 = 0xFD;
// const REG_CTRL: u16 = 0xFE;
// const MSK_CTRL_SLEEP: u16 = 0b00000001;
// const MSK_CTRL_RESET: u16 = 0b00000010;
// const MSK_CTRL_FREAD: u16 = 0b00000100;
// const MSK_CTRL_FWRITE: u16 = 0b00001000;

pub struct Trackball {
    i2c: I2c,
    big_endian: bool,
    contraste:u8
}

impl Trackball {
    pub fn new() -> Result<Self, rppal::i2c::Error> {
        let big_endian;
        if cfg!(target_endian = "big") {
            big_endian = true;
        } else {
            big_endian = false;
        }

        Ok(Self {
            i2c: I2c::new()?,
            big_endian: big_endian,
            contraste:0xff
        })
    }

    /// Turn on the trackball.
    pub fn turn_on(&mut self) -> Result<(), rppal::i2c::Error> {
        self.i2c.set_slave_address(I2C_ADDR_PRIMARY)?;

        let mut chip_id_btyes = [0u8; 2];
        self.i2c
            .block_read(REG_CHIP_ID_L as u8, &mut chip_id_btyes)?;
        let chip_id;
        match self.big_endian {
            true => chip_id = u16::from_be_bytes(chip_id_btyes),
            false => chip_id = u16::from_le_bytes(chip_id_btyes),
        }
        if CHIP_ID != chip_id {
            println!(
                "Trackball chip Not Found. Invalid CHIP ID: 0x${:x}",
                chip_id
            );
        }
        Ok(())
    }

    /// Set the colour of the trackball leds.
    pub fn set_colour(&mut self, r: u8, g: u8, b: u8, w: u8) -> Result<(), rppal::i2c::Error> {
        let contrast;
        match r > 0 || g > 0 || b > 0 || w > 0 {
            false => contrast = 0.0,
            true => contrast = self.contraste as f64 / 255.0,
        }
        let red = (r as f64 * contrast) as u8;
        let green = (g as f64 * contrast) as u8;
        let blue = (b as f64 * contrast) as u8;
        let white = (w as f64 * contrast) as u8;

        self.i2c.block_write(REG_LED_RED, &[red])?;
        self.i2c.block_write(REG_LED_GRN, &[green])?;
        self.i2c.block_write(REG_LED_BLU, &[blue])?;
        self.i2c.block_write(REG_LED_WHT, &[white])?;

        println!("Couleur {} {} {} {}", red, green, blue, white);

        // const contrast = (0 !== this.r + this.g + this.b + this.w) ? this.contrast / 0xff : 0x00;
        // await this.writeByte(constants.REG_LED_RED, Math.round(this.r * contrast));
        // await this.writeByte(constants.REG_LED_GRN, Math.round(this.g * contrast));
        // await this.writeByte(constants.REG_LED_BLU, Math.round(this.b * contrast));
        // await this.writeByte(constants.REG_LED_WHT, Math.round(this.w * contrast));

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut trackball: Trackball = Trackball::new()?;
    trackball.turn_on()?;
    trackball.set_colour(0x00, 0x00, 0xff, 0x00)?;
    println!("Hello, world!");
    Ok(())
}
