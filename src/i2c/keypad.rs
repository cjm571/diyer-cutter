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

pub enum Key {
    One = 0b000000000001,
    Two = 0b000000000010,
    Three = 0b000000000100,
    Four = 0b000000001000,
    Five = 0b000000010000,
    Six = 0b000000100000,
    Seven = 0b000001000000,
    Eight = 0b000010000000,
    Nine = 0b000100000000,
    Star = 0b001000000000,
    Zero = 0b010000000000,
    Pound = 0b100000000000,
}

#[derive(Debug, Default)]
pub struct PressedKeys {
    one: bool,
    two: bool,
    three: bool,
    four: bool,
    five: bool,
    six: bool,
    seven: bool,
    eight: bool,
    nine: bool,
    star: bool,
    zero: bool,
    pound: bool,
}

///////////////////////////////////////////////////////////////////////////////
//  Object Implementations
///////////////////////////////////////////////////////////////////////////////

impl PressedKeys {
    /*  *  *  *  *  *  *  *  *\
     *  Accessors/Mutators   *
    \*  *  *  *  *  *  *  *  */

    pub fn set_key(&mut self, key: Key) {
        match key {
            Key::One => self.one = true,
            Key::Two => self.two = true,
            Key::Three => self.three = true,
            Key::Four  => self.four = true,
            Key::Five  => self.five = true,
            Key::Six => self.six = true,
            Key::Seven => self.seven = true,
            Key::Eight => self.eight = true,
            Key::Nine => self.nine = true,
            Key::Star => self.star = true,
            Key::Zero => self.zero = true,
            Key::Pound => self.pound = true,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
//  Static Functions
///////////////////////////////////////////////////////////////////////////////

pub fn init<T: twim::Instance>(i2c: &mut Twim<T>) {
    // Set row pins on keypad's MCP23008 to Input mode (1), leave columns in Output mode (0)
    register_value_set(I2C_ADDR_KEYPAD, MCP23008Register::IODIR, MASK_ALL_ROWS, i2c);
}

//OPT: Probably a more clever way to do this...
// Sweep across keypad columns and read each row to get button presses
pub fn scan<T: twim::Instance>(i2c: &mut Twim<T>) -> PressedKeys {
    let mut pressed_keys = PressedKeys::default();
    
    // Set C1 High and read Row values for presses
    gpio_write(I2C_ADDR_KEYPAD, MASK_C1, i2c);
    let c1_presses = gpio_read(I2C_ADDR_KEYPAD, i2c);

    // Check for "1" press
    if c1_presses & MASK_R1 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '1' Pressed");
        pressed_keys.set_key(Key::One);
    }
    // Check for "4" press
    if c1_presses & MASK_R2 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '4' Pressed");
        pressed_keys.set_key(Key::Four);
    }
    // Check for "7" press
    if c1_presses & MASK_R3 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '7' Pressed");
        pressed_keys.set_key(Key::Seven);
    }
    // Check for "*" press
    if c1_presses & MASK_R4 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '*' Pressed");
        pressed_keys.set_key(Key::Star);
    }
    
    // Set C2 High and read Row values for presses
    gpio_write(I2C_ADDR_KEYPAD, MASK_C2, i2c);
    let c2_presses = gpio_read(I2C_ADDR_KEYPAD, i2c);

    // Check for "2" press
    if c2_presses & MASK_R1 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '2' Pressed");
        pressed_keys.set_key(Key::Two);
    }
    // Check for "5" press
    if c2_presses & MASK_R2 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '5' Pressed");
        pressed_keys.set_key(Key::Five);
    }
    // Check for "8" press
    if c2_presses & MASK_R3 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '8' Pressed");
        pressed_keys.set_key(Key::Eight);
    }
    // Check for "0" press
    if c2_presses & MASK_R4 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '0' Pressed");
        pressed_keys.set_key(Key::Zero);
    }
    
    // Set C3 High and read Row values for presses
    gpio_write(I2C_ADDR_KEYPAD, MASK_C3, i2c);
    let c3_presses = gpio_read(I2C_ADDR_KEYPAD, i2c);
    
    // Check for "3" press
    if c3_presses & MASK_R1 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '3' Pressed");
        pressed_keys.set_key(Key::Three);
    }
    // Check for "6" press
    if c3_presses & MASK_R2 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '6' Pressed");
        pressed_keys.set_key(Key::Six);
    }
    // Check for "9" press
    if c3_presses & MASK_R3 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '9' Pressed");
        pressed_keys.set_key(Key::Nine);
    }
    // Check for "#" press
    if c3_presses & MASK_R4 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '#' Pressed");
        pressed_keys.set_key(Key::Pound);
    }

    pressed_keys
}