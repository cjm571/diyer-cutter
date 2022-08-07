/* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *\
Copyright (C) 2022 CJ McAllister
    This program is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.
    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.
    You should have received a copy of the GNU General Public License
    along with this program; if not, write to the Free Software Foundation,
    Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301  USA
\* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */

use microbit::{
    hal::{
        gpio::{Output, Pin, PushPull},
        prelude::*,
        Timer, timer, twim, Twim,
    },
};
use rtt_target::rprintln;

use super::{I2C_ADDR_LCD, MCP23008Register};


///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const LCD_MAX_LINE_LENGTH: usize = 16;
const LCD_MAX_NEWLINES: usize = 1;
const ASCII_INT_OFFSET: usize = 48;

const MASK_EN: u8 = 0b00000001;
const MASK_RW: u8 = 0b00000010;
const MASK_RS: u8 = 0b00000100;
const MASK_D0: u8 = 0b00001000;
const MASK_D1: u8 = 0b00010000;
const MASK_D3: u8 = 0b00100000;
const MASK_D4: u8 = 0b01000000;
const MASK_ALL: u8 = 0b01111111;


///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

pub struct Lcd1602 {
    input_pins: LcdInputPins,
}

pub struct LcdInputPins {
    d: [Pin<Output<PushPull>>; 8],
    rs: Pin<Output<PushPull>>,
    rw: Pin<Output<PushPull>>,
    en: Pin<Output<PushPull>>,
}

#[derive(PartialEq)]
enum Direction {
    Left,
    Right,
}


///////////////////////////////////////////////////////////////////////////////
//  Object Implementation
///////////////////////////////////////////////////////////////////////////////

impl Lcd1602 {
    //OPT: *STYLE* Consider following the "take" semantics of the Board struct
    pub fn new(input_pins: LcdInputPins) -> Self {
        Self { input_pins }
    }

    pub fn initialize<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // Wait a little bit before entering main loop
        rprintln!("Giving LCD time to initialize...");
        timer.delay_ms(15_u32);

        // Set up LCD for 8-bit, 2-line mode
        self.input_pins.set_8bit_2line_mode(timer, i2c);
        rprintln!("Setting LCD up for 8bit, 2line mode...");

        // Set up LCD cursor
        self.input_pins.set_cursor(timer, i2c);
        rprintln!("Setting LCD cursor up...");

        // Set up auto-increment
        self.input_pins.set_autoincrement(timer, i2c);
        rprintln!("Setting up auto-increment...");

        // Clear the display before anything is written
        self.input_pins.clear_display(timer, i2c);
        rprintln!("Clearing display...");
    }

    pub fn display_greeting<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // Write "HI BABE!"
        self.input_pins
            .write_string("HI BABE! <3\nYou so pretty...", timer, i2c);
        rprintln!("Writing greeting...");
    }

    //OPT: Implement an "overwrite" option for writing
    pub fn write_string<T: timer::Instance, U: twim::Instance>(&mut self, out_str: &str, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.input_pins.write_string(out_str, timer, i2c);
    }

    pub fn write_u8<T: timer::Instance, U: twim::Instance>(&mut self, val: u8, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // Convert value to its zero-padded ASCII values
        let ones = val % 10;
        let tens = ((val - ones) % 100) / 10;
        let hundreds = (val - tens - ones) / 100;

        let ascii_vals = [
            hundreds + ASCII_INT_OFFSET as u8,
            tens + ASCII_INT_OFFSET as u8,
            ones + ASCII_INT_OFFSET as u8,
        ];

        // Encode ASCII values as &strs
        let mut tmp = [0; 3];
        for i in 0..3 {
            // Write the stringified value to the display
            let out_str = (ascii_vals[i] as char).encode_utf8(&mut tmp);
            self.input_pins.write_string(out_str, timer, i2c);
        }
    }

    //TODO: Fix this weird straddled architecture
    pub fn backspace<T: timer::Instance, U: twim::Instance>(&mut self, count: usize, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.input_pins.backspace(count, timer, i2c);
    }
}

impl LcdInputPins {
    pub fn new(
        d: [Pin<Output<PushPull>>; 8],
        rs: Pin<Output<PushPull>>,
        rw: Pin<Output<PushPull>>,
        en: Pin<Output<PushPull>>,
    ) -> Self {
        Self { d, rs, rw, en }
    }

    pub fn write_string<T: timer::Instance, U: twim::Instance>(&mut self, out_str: &str, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // Sanity-check input
        let lines = out_str.split('\n');
        for (i, line) in lines.enumerate() {
            if line.len() > LCD_MAX_LINE_LENGTH {
                panic!(
                    "Line '{}' exceeds LCD Max Length ({})",
                    line, LCD_MAX_LINE_LENGTH
                );
            }

            if i > LCD_MAX_NEWLINES {
                panic!("Too many newlines for LCD");
            }
        }

        for c in out_str.chars() {
            // Move the cursor on newline, otherwise write out the character
            if c == '\n' {
                self.newline(timer, i2c);
            } else {
                self.write_char(c, timer, i2c);
            }
        }
    }

    pub fn backspace<T: timer::Instance, U: twim::Instance>(&mut self, count: usize, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        for _i in 0..count {
            // Shift cursor backwards
            self.shift_cursor(Direction::Left, timer, i2c);

            // Write a blank character code
            self.write_char(32 as char, timer, i2c);

            // Shift cursor backwards again in prep for next char entry
            self.shift_cursor(Direction::Left, timer, i2c);
        }
    }

    fn pulse_enable<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // Delay before setting Enable high to ensure that Address Settling time (60ns) is not violated
        timer.delay_us(1000_u32);

        // Set EN high
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_EN, i2c);

        // Enable must be held high for at least 500ns (at 3.3V operation) per HD44780U datasheet
        // However, in practice holding for just 500us was unstable, so hold for 1000us
        timer.delay_us(1000_u32);

        // Set EN low
        rmw_mask_val_unset(MCP23008Register::GPIO, MASK_EN, i2c);

        // Delay another 1us to ensure Enable Cycle Time minimum (1000ns) is not violated
        timer.delay_us(1000_u32);
    }

    pub fn reset_pins<U: twim::Instance>(&mut self, i2c: &mut Twim<U>) {
        rmw_mask_val_unset(MCP23008Register::GPIO, MASK_ALL, i2c)
    }

    //FIXME: Update to I2C
    pub fn clear_display<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.reset_pins(i2c);

        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D0, i2c);

        self.pulse_enable(timer, i2c);
    }

    //FIXME: Update to I2C
    pub fn set_8bit_2line_mode<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.reset_pins(i2c);

        self.d[5].set_high().unwrap();
        self.d[4].set_high().unwrap();
        self.d[3].set_high().unwrap();

        self.pulse_enable(timer, i2c);
    }

    //FIXME: Update to I2C
    pub fn set_cursor<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.reset_pins(i2c);

        self.d[3].set_high().unwrap();
        self.d[2].set_high().unwrap();
        self.d[1].set_high().unwrap();

        self.pulse_enable(timer, i2c);
    }

    //FIXME: Update to I2C
    pub fn set_autoincrement<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.reset_pins(i2c);

        self.d[2].set_high().unwrap();
        self.d[1].set_high().unwrap();

        self.pulse_enable(timer, i2c);
    }

    //FIXME: Update to I2C
    fn write_char<T: timer::Instance, U: twim::Instance>(&mut self, c: char, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.reset_pins(i2c);
        self.rs.set_high().unwrap();

        // Get the ASCII index of the character
        let ascii_idx = c as u32;

        // Check each bit's value and set in the corresponding data bit pin
        for i in 0..=7 {
            if ascii_idx & (1 << i) != 0 {
                self.d[i].set_high().unwrap();
            }
        }

        self.pulse_enable(timer, i2c);
    }

    //FIXME: Update to I2C
    fn shift_cursor<T: timer::Instance, U: twim::Instance>(&mut self, dir: Direction, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.reset_pins(i2c);

        self.d[4].set_high().unwrap();

        // Left == low, Right == high
        if dir == Direction::Right {
            self.d[2].set_high().unwrap();
        }

        self.pulse_enable(timer, i2c);
    }

    //FIXME: Update to I2C
    fn newline<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.reset_pins(i2c);

        self.d[7].set_high().unwrap();
        self.d[6].set_high().unwrap();

        self.pulse_enable(timer, i2c);
    }

    #[allow(dead_code)]
    pub fn print_state(&self) {
        rprintln!(
            "rs rw d7 d6 d5 d4 d3 d2 d1 d0 en\n{}  {}  {}  {}  {}  {}  {}  {}  {}  {}  {}",
            self.rs.is_set_high().unwrap() as u32,
            self.rw.is_set_high().unwrap() as u32,
            self.d[7].is_set_high().unwrap() as u32,
            self.d[6].is_set_high().unwrap() as u32,
            self.d[5].is_set_high().unwrap() as u32,
            self.d[4].is_set_high().unwrap() as u32,
            self.d[3].is_set_high().unwrap() as u32,
            self.d[2].is_set_high().unwrap() as u32,
            self.d[1].is_set_high().unwrap() as u32,
            self.d[0].is_set_high().unwrap() as u32,
            self.en.is_set_high().unwrap() as u32,
        );
    }
}


///////////////////////////////////////////////////////////////////////////////
//  Helper Functions
///////////////////////////////////////////////////////////////////////////////

fn rmw_mask_val_set<U: twim::Instance>(reg_addr: MCP23008Register, mask_val: u8, i2c: &mut Twim<U>) {
    // Read value current in specified register
    let mut rd_buffer: [u8;1] = [0x00];
    i2c.write_then_read(I2C_ADDR_LCD, &[reg_addr as u8], &mut rd_buffer).unwrap();

    // Modify the read value with mask
    let modified_data = rd_buffer[0] | mask_val;

    // Write the modified value back
    let reg_addr_and_data: [u8; 2] = [reg_addr as u8, modified_data]; 
    i2c.write(I2C_ADDR_LCD, &reg_addr_and_data).unwrap();
}

fn rmw_mask_val_unset<U: twim::Instance>(reg_addr: MCP23008Register, mask_val: u8, i2c: &mut Twim<U>) {
    // Read value current in specified register
    let mut rd_buffer: [u8;1] = [0x00];
    i2c.write_then_read(I2C_ADDR_LCD, &[reg_addr as u8], &mut rd_buffer).unwrap();

    // Modify the read value with mask
    let modified_data = rd_buffer[0] & !mask_val;

    // Write the modified value back
    let reg_addr_and_data: [u8; 2] = [reg_addr as u8, modified_data]; 
    i2c.write(I2C_ADDR_LCD, &reg_addr_and_data).unwrap();
}