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

use super::*;


///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const MASK_C2: u8 = 0b00000001;
const MASK_R1: u8 = 0b00000010;
const MASK_C1: u8 = 0b00000100;
const MASK_R4: u8 = 0b00001000;
const MASK_C3: u8 = 0b00010000;
const MASK_R3: u8 = 0b00100000;
const MASK_R2: u8 = 0b01000000;

#[allow(dead_code)]
const MASK_ALL_COLS: u8 = MASK_C1 | MASK_C2 | MASK_C3;
const MASK_ALL_ROWS: u8 = MASK_R1 | MASK_R2 | MASK_R3 | MASK_R4;


///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

pub enum Keys {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Star,
    Zero,
    Pound,
}

///////////////////////////////////////////////////////////////////////////////
//  Static Functions
///////////////////////////////////////////////////////////////////////////////

pub fn init<T: twim::Instance>(i2c: &mut Twim<T>) {
    // Set row pins on keypad's MCP23008 to Input mode (1), leave columns in Output mode (0)
    register_value_set(I2C_ADDR_KEYPAD, MCP23008Register::IODIR, MASK_ALL_ROWS, i2c);
}

// Sweep across keypad columns and read each row to get button presses
pub fn scan<T: twim::Instance>(i2c: &mut Twim<T>) {
    // Set C1 High in prep for row scan
    gpio_write(I2C_ADDR_KEYPAD, MASK_C1, i2c);
    let presses = scan_rows(i2c);
}


///////////////////////////////////////////////////////////////////////////////
//  Helper Functions
///////////////////////////////////////////////////////////////////////////////

fn scan_rows<T: twim::Instance>(i2c: &mut Twim<T>) {
    
}