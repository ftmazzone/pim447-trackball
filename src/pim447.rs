use rppal::i2c::I2c;

const I2C_ADDR_PRIMARY: u16 = 0x0A;
//const I2C_ADDR_ALTERNATIVE: u16 = 0x0B;
const CHIP_ID: u16 = 0xBA11;
//const VERSION: u16 = 1;
const REG_LED_RED: u8 = 0x00;
const REG_LED_GRN: u8 = 0x01;
const REG_LED_BLU: u8 = 0x02;
const REG_LED_WHT: u8 = 0x03;
const REG_LEFT: u8 = 0x04;
// const REG_RIGHT: u16 = 0x05;
// const REG_UP: u16 = 0x06;
// const REG_DOWN: u16 = 0x07;
// const REG_SWITCH: u16 = 0x08;
const MSK_CLICKED: u8 = 0x80;
const MSK_CLICK_STATE_UPDATE: u8 = 0x01;
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
    contrast: u8,
    red: u8,
    green: u8,
    blue: u8,
    white: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct Input {
    pub x: i16,
    pub y: i16,
    pub clicked: u8,
    pub click_state_update: u8,
    pub state_update: bool,
}

pub enum Command {
    TurnOff,
    TurnOn,
    SetColour(u8, u8, u8, u8),
    SetContrast(u8),
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
            contrast: 0x00,
            green: 0x00,
            red: 0x00,
            blue: 0x00,
            white: 0x00,
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

    /// Turn off the trackball.
    pub fn turn_off(&mut self) -> Result<(), rppal::i2c::Error> {
        self.set_contrast(0x00)?;
        Ok(())
    }

    // Set the contrast of the trackball leds.
    pub fn set_contrast(&mut self, contrast: u8) -> Result<(), rppal::i2c::Error> {
        self.contrast = contrast;
        self.set_colour(self.red, self.green, self.blue, self.white)
    }

    /// Set the colour of the trackball leds.
    pub fn set_colour(&mut self, r: u8, g: u8, b: u8, w: u8) -> Result<(), rppal::i2c::Error> {
        let contrast;

        self.red = r;
        self.green = g;
        self.blue = b;
        self.white = w;

        match self.red > 0 || self.green > 0 || self.blue > 0 || self.white > 0 {
            false => contrast = 0.0,
            true => contrast = self.contrast as f64 / 255.0,
        }
        let red = (self.red as f64 * contrast) as u8;
        let green = (self.green as f64 * contrast) as u8;
        let blue = (self.blue as f64 * contrast) as u8;
        let white = (self.white as f64 * contrast) as u8;

        self.i2c.block_write(REG_LED_RED, &[red])?;
        self.i2c.block_write(REG_LED_GRN, &[green])?;
        self.i2c.block_write(REG_LED_BLU, &[blue])?;
        self.i2c.block_write(REG_LED_WHT, &[white])?;

        Ok(())
    }

    /// Read the trackball inputs
    pub fn read_input(&mut self) -> Result<Input, rppal::i2c::Error> {
        let mut raw_inputs_bytes = [0u8; 5];
        self.i2c.block_read(REG_LEFT as u8, &mut raw_inputs_bytes)?;
        let left: u8 = raw_inputs_bytes[0];
        let right = raw_inputs_bytes[1];
        let up = raw_inputs_bytes[2];
        let down = raw_inputs_bytes[3];
        let clicked = raw_inputs_bytes[4] & MSK_CLICKED;
        let click_state_update = !!raw_inputs_bytes[4] & MSK_CLICK_STATE_UPDATE;

        let state_update;
        if left > 0 || right > 0 || up > 0 || down > 0 || click_state_update > 0 {
            state_update = true;
        } else {
            state_update = false;
        }

        let input: Input = Input {
            x: (right as i16 - left as i16),
            y: (down as i16 - up as i16),
            clicked: clicked,
            click_state_update: click_state_update,
            state_update: state_update,
        };

        Ok(input)
    }
}
