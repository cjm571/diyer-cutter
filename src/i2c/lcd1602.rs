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

use super::*;

///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const LCD_MAX_LINE_LENGTH: usize = 16;
const LCD_MAX_NEWLINES: usize = 1;
#[allow(dead_code)]
const ASCII_INT_OFFSET: usize = 48;

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

/*  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *\
 *   5V Bus Timing Characteristics, per HD44789U datasheet   *
\*  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *  *  */

// Power supply rise time
const T_RCC_IN_MS: u32 = 10;
// Address set-up time (RS, R/W to E)
const T_AS_IN_US: u32 = 40;
// Enable pulse width (high level)
const PW_EH_IN_US: u32 = 230;
// Enable cycle time
const T_CYCE_IN_US: u32 = 500;

///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
#[derive(PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

///////////////////////////////////////////////////////////////////////////////
//  Static Functions
///////////////////////////////////////////////////////////////////////////////

pub fn power_on<U: twim::Instance>(i2c: &mut Twim<U>) {
    gpio_set_rmw(I2C_ADDR_LCD, MASK_PWR, i2c);
}

#[allow(dead_code)]
pub fn power_off<U: twim::Instance>(i2c: &mut Twim<U>) {
    gpio_unset_rmw(I2C_ADDR_LCD, MASK_PWR, i2c);
}

pub fn init<T: timer::Instance, U: twim::Instance>(timer: &mut Timer<T>, i2c: &mut Twim<U>) {
    // 0. Set all pins on LCD Display's MCP23008 to Output mode (0)
    register_value_set(I2C_ADDR_LCD, MCP23008Register::IODIR, 0b00000000, i2c);

    // 1. Allow time for LCD VCC to rise to 4.5V
    defmt::println!("Giving LCD time to initialize...");
    timer.delay_ms(T_RCC_IN_MS);

    // 2, 3. Set up LCD for 4-bit operation, 2-line Mode
    defmt::println!("Setting LCD up for 4bit Operation, 2-Line Mode...");
    set_4bit_2line_mode(timer, i2c);

    // 4. Turn on display/cursor
    defmt::println!("Turning on LCD Display and Cursor...");
    set_cursor(timer, i2c);

    // 5. Entry mode set
    defmt::println!("Setting entry mode to INCR, no SHIFT...");
    set_autoincrement(timer, i2c);

    defmt::println!("LCD Initialization Complete");
}

pub fn display_greeting<T: timer::Instance, U: twim::Instance>(
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    // Write "HI BABE!"
    write_string("HI BABE! <3\nYou so pretty...", timer, i2c);
    defmt::println!("Writing greeting...");
}

pub fn write_u32<T: timer::Instance, U: twim::Instance>(
    val: u32,
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    // Convert value to its zero-padded ASCII values
    let ones = val % 10;
    let tens = ((val - ones) % 100) / 10;
    let hundreds = ((val - tens - ones) % 1000) / 100;
    let thousands = ((val - hundreds - tens - ones) % 10000) / 1000;
    let ten_thousands = (val - thousands - hundreds - tens - ones) / 10000;

    let ascii_vals = [
        ten_thousands + ASCII_INT_OFFSET as u32,
        thousands + ASCII_INT_OFFSET as u32,
        hundreds + ASCII_INT_OFFSET as u32,
        tens + ASCII_INT_OFFSET as u32,
        ones + ASCII_INT_OFFSET as u32,
    ];

    // Encode ASCII values as &strs
    let mut tmp = [0; 5];
    for ascii_val in ascii_vals {
        // Write the stringified value to the display
        let out_str = (char::from_u32(ascii_val)).unwrap().encode_utf8(&mut tmp);
        write_string(out_str, timer, i2c);
    }
}

//FEAT: Implement an "overwrite" option for writing
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
        shift_cursor(Direction::Left, 1, timer, i2c);

        // Write a blank character code
        write_char(32 as char, timer, i2c);

        // Shift cursor backwards again in prep for next char entry
        shift_cursor(Direction::Left, 1, timer, i2c);
    }
}

fn pulse_enable<T: timer::Instance, U: twim::Instance>(timer: &mut Timer<T>, i2c: &mut Twim<U>) {
    // Delay before setting EN high to ensure that Address Set-Up time is not violated
    timer.delay_us(T_AS_IN_US);

    // Set EN high
    gpio_set_rmw(I2C_ADDR_LCD, MASK_EN, i2c);

    // Hold EN high for the required time
    timer.delay_us(PW_EH_IN_US);

    // Set EN low
    gpio_unset_rmw(I2C_ADDR_LCD, MASK_EN, i2c);

    // Delay before allowing other operations to ensure Enable cycle time is not violated
    timer.delay_us(T_CYCE_IN_US - PW_EH_IN_US);
}

pub fn reset_pins<U: twim::Instance>(i2c: &mut Twim<U>) {
    gpio_unset_rmw(I2C_ADDR_LCD, MASK_ALL, i2c);
}

#[allow(dead_code)]
pub fn clear_display<T: timer::Instance, U: twim::Instance>(
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    // Higher-order data bits write
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_NONE, i2c);
    pulse_enable(timer, i2c);

    // Lower-order data bits write
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_D4, i2c);
    pulse_enable(timer, i2c);
}

pub fn set_4bit_2line_mode<T: timer::Instance, U: twim::Instance>(
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    // First phase of Function Set command - sets 4-bit operation mode (just one write, unlike most others)
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_D5, i2c);
    pulse_enable(timer, i2c);

    // Second phase of Function Set command - sets 4-bit, 2-line mode
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_D5, i2c);
    pulse_enable(timer, i2c);
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_D7, i2c);
    pulse_enable(timer, i2c);
}

pub fn set_cursor<T: timer::Instance, U: twim::Instance>(timer: &mut Timer<T>, i2c: &mut Twim<U>) {
    // Higher-order data bits write
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_NONE, i2c);
    pulse_enable(timer, i2c);

    // Lower-order data bits write
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_D5 | MASK_D6 | MASK_D7, i2c);
    pulse_enable(timer, i2c);
}

pub fn set_autoincrement<T: timer::Instance, U: twim::Instance>(
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    // Higher-order data bits write
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_NONE, i2c);
    pulse_enable(timer, i2c);

    // Lower-order data bits write
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_D5 | MASK_D6, i2c);
    pulse_enable(timer, i2c);
}

fn write_char<T: timer::Instance, U: twim::Instance>(
    c: char,
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_RS, i2c);

    // Get the ASCII index of the character
    let ascii_idx = c as u32;

    // Calculate higher-order bit mask based on ascii index value, set pins accordingly and pulse enable
    let hi_order_mask = ((ascii_idx & (1 << 4)
        | ascii_idx & (1 << 5)
        | ascii_idx & (1 << 6)
        | ascii_idx & (1 << 7))
        >> 1) as u8;
    gpio_set_rmw(I2C_ADDR_LCD, hi_order_mask, i2c);
    pulse_enable(timer, i2c);

    // Calculate lower-order bit mask based on ascii index value, set pins accordingly and pulse enable
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_RS, i2c);
    let lo_order_mask = ((ascii_idx & (1 << 0)
        | ascii_idx & (1 << 1)
        | ascii_idx & (1 << 2)
        | ascii_idx & (1 << 3))
        << 3) as u8;
    gpio_set_rmw(I2C_ADDR_LCD, lo_order_mask, i2c);
    pulse_enable(timer, i2c);
}

pub fn shift_cursor<T: timer::Instance, U: twim::Instance>(
    dir: Direction,
    num_spaces: usize,
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) {
    for _ in 0..num_spaces {
        // Higher-order data bits write
        reset_pins(i2c);
        gpio_set_rmw(I2C_ADDR_LCD, MASK_D4, i2c);
        pulse_enable(timer, i2c);

        // Lower-order data bits write
        reset_pins(i2c);
        // Left == low, Right == high
        if dir == Direction::Right {
            gpio_set_rmw(I2C_ADDR_LCD, MASK_D6, i2c);
        }
        pulse_enable(timer, i2c);
    }
}

fn newline<T: timer::Instance, U: twim::Instance>(timer: &mut Timer<T>, i2c: &mut Twim<U>) {
    // Higher-order data bits write
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_D6 | MASK_D7, i2c);
    pulse_enable(timer, i2c);

    // Lower-order data bits write
    reset_pins(i2c);
    gpio_set_rmw(I2C_ADDR_LCD, MASK_NONE, i2c);
    pulse_enable(timer, i2c);
}
