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

use microbit::hal::{timer, twim, Timer, Twim};

#[cfg(feature = "debug_keypad")]
use rtt_target::rprintln;

use super::*;

///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const DEBOUNCE_DELAY_IN_US: u32 = 250;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Key {
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
//  Object Implementations
///////////////////////////////////////////////////////////////////////////////

impl From<Key> for &str {
    fn from(key: Key) -> Self {
        match key {
            Key::One => "1",
            Key::Two => "2",
            Key::Three => "3",
            Key::Four => "4",
            Key::Five => "5",
            Key::Six => "6",
            Key::Seven => "7",
            Key::Eight => "8",
            Key::Nine => "9",
            Key::Star => "*",
            Key::Zero => "0",
            Key::Pound => "#",
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
pub fn scan<T: timer::Instance, U: twim::Instance>(
    timer: &mut Timer<T>,
    i2c: &mut Twim<U>,
) -> Option<Key> {
    let mut pressed_key = None;
    let mut key_pressed = false;

    // Set C1 High and read Row values for presses
    gpio_write(I2C_ADDR_KEYPAD, MASK_C1, i2c);
    let c1_presses = gpio_read(I2C_ADDR_KEYPAD, i2c);

    // Check for "1" press
    if c1_presses & MASK_R1 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '1' Pressed");
        pressed_key = Some(Key::One);
        key_pressed = true;
    }
    // Check for "4" press
    if c1_presses & MASK_R2 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '4' Pressed");
        pressed_key = Some(Key::Four);
        key_pressed = true;
    }
    // Check for "7" press
    if c1_presses & MASK_R3 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '7' Pressed");
        pressed_key = Some(Key::Seven);
        key_pressed = true;
    }
    // Check for "*" press
    if c1_presses & MASK_R4 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '*' Pressed");
        pressed_key = Some(Key::Star);
        key_pressed = true;
    }

    // Set C2 High and read Row values for presses
    gpio_write(I2C_ADDR_KEYPAD, MASK_C2, i2c);
    let c2_presses = gpio_read(I2C_ADDR_KEYPAD, i2c);

    // Check for "2" press
    if c2_presses & MASK_R1 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '2' Pressed");
        pressed_key = Some(Key::Two);
        key_pressed = true;
    }
    // Check for "5" press
    if c2_presses & MASK_R2 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '5' Pressed");
        pressed_key = Some(Key::Five);
        key_pressed = true;
    }
    // Check for "8" press
    if c2_presses & MASK_R3 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '8' Pressed");
        pressed_key = Some(Key::Eight);
        key_pressed = true;
    }
    // Check for "0" press
    if c2_presses & MASK_R4 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '0' Pressed");
        pressed_key = Some(Key::Zero);
        key_pressed = true;
    }

    // Set C3 High and read Row values for presses
    gpio_write(I2C_ADDR_KEYPAD, MASK_C3, i2c);
    let c3_presses = gpio_read(I2C_ADDR_KEYPAD, i2c);

    // Check for "3" press
    if c3_presses & MASK_R1 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '3' Pressed");
        pressed_key = Some(Key::Three);
        key_pressed = true;
    }
    // Check for "6" press
    if c3_presses & MASK_R2 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '6' Pressed");
        pressed_key = Some(Key::Six);
        key_pressed = true;
    }
    // Check for "9" press
    if c3_presses & MASK_R3 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '9' Pressed");
        pressed_key = Some(Key::Nine);
        key_pressed = true;
    }
    // Check for "#" press
    if c3_presses & MASK_R4 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '#' Pressed");
        pressed_key = Some(Key::Pound);
        key_pressed = true;
    }

    if key_pressed {
        //OPT: This debouncing implementation will blow up if a key is held for too long...
        // Key was pressed, to "debounce" scan until it's no longer pressed
        while let Some(_still_pressed_key) = scan(timer, i2c) {
            #[cfg(feature = "debug_keypad")]
            rprintln!("DEBUG_KEYPAD: Debouncing '{:?}'...", _still_pressed_key);
            timer.delay_us(DEBOUNCE_DELAY_IN_US);
        }

        // Return the pressed key
        pressed_key
    } else {
        // No key was pressed, return None
        None
    }
}
