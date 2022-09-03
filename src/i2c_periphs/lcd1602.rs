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

use microbit::hal::{prelude::*, timer, twim, Timer, Twim};
use rtt_target::rprintln;

use super::{MCP23008Register, I2C_ADDR_LCD};


///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const LCD_MAX_LINE_LENGTH: usize = 16;
const LCD_MAX_NEWLINES: usize = 1;
#[allow(dead_code)]
const ASCII_INT_OFFSET: usize = 48;

const GPIO_REG_ADDR: u8 = MCP23008Register::GPIO as u8;

const MASK_RS: u8 = 0b00000001;
#[allow(dead_code)]
const MASK_RW: u8 = 0b00000010;
const MASK_EN: u8 = 0b00000100;
const MASK_D4: u8 = 0b00001000;
const MASK_D5: u8 = 0b00010000;
const MASK_D6: u8 = 0b00100000;
const MASK_D7: u8 = 0b01000000;
const MASK_ALL: u8 = 0b01111111;
const MASK_NONE: u8 = 0b00000000;
const MASK_PWR: u8 = 0b10000000;

const EN_PULSE_WIDTH: u32 = 500;


///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
#[derive(PartialEq)]
enum Direction {
    Left,
    Right,
}


///////////////////////////////////////////////////////////////////////////////
//  Static Functions
///////////////////////////////////////////////////////////////////////////////

pub fn power_on<U: twim::Instance>(i2c: &mut Twim<U>) {
    rmw_mask_val_set(MASK_PWR, i2c);
}

#[allow(dead_code)]
pub fn power_off<U: twim::Instance>(i2c: &mut Twim<U>) {
    rmw_mask_val_unset(MASK_PWR, i2c);
}

pub fn initialize<T: timer::Instance, U: twim::Instance>(timer: &mut Timer<T>, i2c: &mut Twim<U>) {
    // 1. Allow time for LCD VCC to rise to 4.5V
    rprintln!("Giving LCD time to initialize...");
    timer.delay_ms(10_u32);

    // 2, 3. Set up LCD for 4-bit operation, 2-line Mode
    rprintln!("Setting LCD up for 4bit Operation, 2-Line Mode...");
    set_4bit_2line_mode(timer, i2c);

    // 4. Turn on display/cursor
    rprintln!("Turning on LCD Display and Cursor...");
    set_cursor(timer, i2c);

    // 5. Entry mode set
    rprintln!("Setting entry mode to INCR, no SHIFT...");
    set_autoincrement(timer, i2c);

    rprintln!("LCD Initialization Complete");
}

pub fn display_greeting<T: timer::Instance, U: twim::Instance>(
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    // Write "HI BABE!"
    write_string("HI BABE! <3\nYou so pretty...", timer, i2c);
    rprintln!("Writing greeting...");
}

#[allow(dead_code)]
pub fn write_u8<T: timer::Instance, U: twim::Instance>(
    val: u8,
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
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
        write_string(out_str, timer, i2c);
    }
}

//OPT: Implement an "overwrite" option for writing
pub fn write_string<T: timer::Instance, U: twim::Instance>(
    out_str: &str,
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
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
            newline(timer, i2c);
        } else {
            write_char(c, timer, i2c);
        }
    }
}

#[allow(dead_code)]
pub fn backspace<T: timer::Instance, U: twim::Instance>(
    count: usize,
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    for _i in 0..count {
        // Shift cursor backwards
        shift_cursor(Direction::Left, timer, i2c);

        // Write a blank character code
        write_char(32 as char, timer, i2c);

        // Shift cursor backwards again in prep for next char entry
        shift_cursor(Direction::Left, timer, i2c);
    }
}

fn pulse_enable<T: timer::Instance, U: twim::Instance>(timer: &mut Timer<T>, i2c: &mut Twim<U>) {
    // Delay before setting EN high to ensure that Address Set-Up time (tAS, 40ms) is not violated
    timer.delay_us(40_u32);

    // Set EN high
    rmw_mask_val_set(MASK_EN, i2c);

    // Hold EN high for the required time from the datasheet (PWEH, 230ms)
    timer.delay_us(230_u32);

    // Set EN low
    rmw_mask_val_unset(MASK_EN, i2c);

    // Delay before allowing other operations (remainder of tcycE, 270ms)
    timer.delay_us(270_u32);
}

pub fn reset_pins<U: twim::Instance>(i2c: &mut Twim<U>) {
    rmw_mask_val_unset(MASK_ALL, i2c);
}

#[allow(dead_code)]
pub fn clear_display<T: timer::Instance, U: twim::Instance>(
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    // Higher-order data bits write
    reset_pins(i2c);
    rmw_mask_val_set(MASK_NONE, i2c);
    pulse_enable(timer, i2c);

    // Lower-order data bits write
    reset_pins(i2c);
    rmw_mask_val_set(MASK_D4, i2c);
    pulse_enable(timer, i2c);
}

pub fn set_4bit_2line_mode<T: timer::Instance, U: twim::Instance>(
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    // First phase of Function Set command - sets 4-bit operation mode (just one write, unlike most others)
    reset_pins(i2c);
    rmw_mask_val_set(MASK_D5, i2c);
    pulse_enable(timer, i2c);

    // Second phase of Function Set command - sets 4-bit, 2-line mode
    reset_pins(i2c);
    rmw_mask_val_set(MASK_D5, i2c);
    pulse_enable(timer, i2c);
    reset_pins(i2c);
    rmw_mask_val_set(MASK_D7, i2c);
    pulse_enable(timer, i2c);
}

pub fn set_cursor<T: timer::Instance, U: twim::Instance>(timer: &mut Timer<T>, i2c: &mut Twim<U>) {
    // Higher-order data bits write
    reset_pins(i2c);
    rmw_mask_val_set(MASK_NONE, i2c);
    pulse_enable(timer, i2c);

    // Lower-order data bits write
    reset_pins(i2c);
    rmw_mask_val_set(MASK_D5 | MASK_D6 | MASK_D7, i2c);
    pulse_enable(timer, i2c);
}

pub fn set_autoincrement<T: timer::Instance, U: twim::Instance>(
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    // Higher-order data bits write
    reset_pins(i2c);
    rmw_mask_val_set(MASK_NONE, i2c);
    pulse_enable(timer, i2c);

    // Lower-order data bits write
    reset_pins(i2c);
    rmw_mask_val_set(MASK_D5 | MASK_D6, i2c);
    pulse_enable(timer, i2c);
}

fn write_char<T: timer::Instance, U: twim::Instance>(
    c: char,
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    reset_pins(i2c);
    rmw_mask_val_set(MASK_RS, i2c);

    // Get the ASCII index of the character
    let ascii_idx = c as u32;

    // Check each higher-order bit's value and set in the corresponding data bit pin
    //TODO: This can very likely be optimized by using the shifted value directly
    if ascii_idx & (1 << 4) != 0 {
        rmw_mask_val_set(MASK_D4, i2c);
    }
    if ascii_idx & (1 << 5) != 0 {
        rmw_mask_val_set(MASK_D5, i2c);
    }
    if ascii_idx & (1 << 6) != 0 {
        rmw_mask_val_set(MASK_D6, i2c);
    }
    if ascii_idx & (1 << 7) != 0 {
        rmw_mask_val_set(MASK_D7, i2c);
    }
    pulse_enable(timer, i2c);

    // Check each lower-order bit's value and set in the corresponding data bit pin
    reset_pins(i2c);
    rmw_mask_val_set(MASK_RS, i2c);
    //TODO: This can very likely be optimized by using the shifted value directly
    if ascii_idx & (1 << 0) != 0 {
        rmw_mask_val_set(MASK_D4, i2c);
    }
    if ascii_idx & (1 << 1) != 0 {
        rmw_mask_val_set(MASK_D5, i2c);
    }
    if ascii_idx & (1 << 2) != 0 {
        rmw_mask_val_set(MASK_D6, i2c);
    }
    if ascii_idx & (1 << 3) != 0 {
        rmw_mask_val_set(MASK_D7, i2c);
    }
    pulse_enable(timer, i2c);
}

fn shift_cursor<T: timer::Instance, U: twim::Instance>(
    dir: Direction,
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    // Higher-order data bits write
    reset_pins(i2c);
    rmw_mask_val_set(MASK_D4, i2c);
    pulse_enable(timer, i2c);

    // Lower-order data bits write
    reset_pins(i2c);
    // Left == low, Right == high
    if dir == Direction::Right {
        rmw_mask_val_set(MASK_D6, i2c);
    }
    pulse_enable(timer, i2c);
}

fn newline<T: timer::Instance, U: twim::Instance>(timer: &mut Timer<T>, i2c: &mut Twim<U>) {
    // Higher-order data bits write
    reset_pins(i2c);
    rmw_mask_val_set(MASK_D6 | MASK_D7, i2c);
    pulse_enable(timer, i2c);

    // Lower-order data bits write
    reset_pins(i2c);
    rmw_mask_val_set(MASK_NONE, i2c);
    pulse_enable(timer, i2c);
}


///////////////////////////////////////////////////////////////////////////////
//  Helper Functions
///////////////////////////////////////////////////////////////////////////////

fn rmw_mask_val_set<U: twim::Instance>(mask_val: u8, i2c: &mut Twim<U>) {
    // Must declare this locally or the I2C driver will panic
    let gpio_reg_addr = GPIO_REG_ADDR;

    // Read value current in specified register
    let mut rd_buffer: [u8; 1] = [0x00];
    i2c.write_then_read(I2C_ADDR_LCD, &[gpio_reg_addr], &mut rd_buffer)
        .unwrap();

    // Modify the read value with mask
    let modified_data = rd_buffer[0] | mask_val;

    // Write the modified value back
    let reg_addr_and_data: [u8; 2] = [gpio_reg_addr, modified_data];
    i2c.write(I2C_ADDR_LCD, &reg_addr_and_data).unwrap();
}

fn rmw_mask_val_unset<U: twim::Instance>(mask_val: u8, i2c: &mut Twim<U>) {
    // Must declare this locally or the I2C driver will panic
    let gpio_reg_addr = GPIO_REG_ADDR;

    // Read value current in specified register
    let mut rd_buffer: [u8; 1] = [0x00];
    i2c.write_then_read(I2C_ADDR_LCD, &[gpio_reg_addr], &mut rd_buffer)
        .unwrap();

    // Modify the read value with mask
    let modified_data = rd_buffer[0] & !mask_val;

    // Write the modified value back
    let reg_addr_and_data: [u8; 2] = [gpio_reg_addr, modified_data];
    i2c.write(I2C_ADDR_LCD, &reg_addr_and_data).unwrap();
}
