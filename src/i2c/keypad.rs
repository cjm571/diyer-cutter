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

use microbit::hal::{twim, Twim};

#[cfg(feature = "debug_keypad")]
use rtt_target::rprintln;

use super::*;


///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const NUM_KEYS: usize = 12;

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

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Debug)]
pub struct PressedKeys {
    key_states: [(Key, bool); NUM_KEYS],
    itr_idx: usize,
}

///////////////////////////////////////////////////////////////////////////////
//  Object Implementations
///////////////////////////////////////////////////////////////////////////////

impl Into<&str> for Key {
    fn into(self) -> &'static str {
        match self {
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

impl PressedKeys {
    pub fn new() -> Self {
        Self {
            key_states: [
                (Key::One, false),
                (Key::Two, false),
                (Key::Three, false),
                (Key::Four, false),
                (Key::Five, false),
                (Key::Six, false),
                (Key::Seven, false),
                (Key::Eight, false),
                (Key::Nine, false),
                (Key::Star, false),
                (Key::Zero, false),
                (Key::Pound, false),
            ],
            itr_idx: 0,
        }
    }

    /*  *  *  *  *  *  *  *  *\
     *  Accessors/Mutators   *
    \*  *  *  *  *  *  *  *  */

    pub fn set_key(&mut self, key: Key) {
        match key {
            Key::One => self.key_states[0].1 = true,
            Key::Two => self.key_states[1].1 = true,
            Key::Three => self.key_states[2].1 = true,
            Key::Four => self.key_states[3].1 = true,
            Key::Five => self.key_states[4].1 = true,
            Key::Six => self.key_states[5].1 = true,
            Key::Seven => self.key_states[6].1 = true,
            Key::Eight => self.key_states[7].1 = true,
            Key::Nine => self.key_states[8].1 = true,
            Key::Star => self.key_states[9].1 = true,
            Key::Zero => self.key_states[10].1 = true,
            Key::Pound => self.key_states[11].1 = true,
        }
    }
}

impl Iterator for PressedKeys {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        // Bounds pre-check
        if self.itr_idx >= NUM_KEYS {
            #[cfg(feature = "debug_keypad")]
            rprintln!("DEBUG_KEYPAD: Bounds pre-check failed, returning None");
            return None;
        }

        while self.key_states[self.itr_idx].1 == false {
            #[cfg(feature = "debug_keypad")]
            rprintln!(
                "DEBUG_KEYPAD: No pressed keys at itr_idx {}, incrementing",
                self.itr_idx
            );
            self.itr_idx += 1;

            // Bounds check
            if self.itr_idx >= NUM_KEYS {
                #[cfg(feature = "debug_keypad")]
                rprintln!("DEBUG_KEYPAD: Bounds check failed, returning None");
                return None;
            }
        }

        #[cfg(feature = "debug_keypad")]
        rprintln!(
            "DEBUG_KEYPAD: Found pressed key at itr_idx {}",
            self.itr_idx
        );
        let found_idx = self.itr_idx;
        self.itr_idx += 1;
        Some(self.key_states[found_idx].0)
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
pub fn scan<T: twim::Instance>(i2c: &mut Twim<T>) -> Option<PressedKeys> {
    let mut pressed_keys = PressedKeys::new();
    let mut key_pressed = false;

    // Set C1 High and read Row values for presses
    gpio_write(I2C_ADDR_KEYPAD, MASK_C1, i2c);
    let c1_presses = gpio_read(I2C_ADDR_KEYPAD, i2c);

    // Check for "1" press
    if c1_presses & MASK_R1 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '1' Pressed");
        pressed_keys.set_key(Key::One);
        key_pressed = true;
    }
    // Check for "4" press
    if c1_presses & MASK_R2 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '4' Pressed");
        pressed_keys.set_key(Key::Four);
        key_pressed = true;
    }
    // Check for "7" press
    if c1_presses & MASK_R3 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '7' Pressed");
        pressed_keys.set_key(Key::Seven);
        key_pressed = true;
    }
    // Check for "*" press
    if c1_presses & MASK_R4 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '*' Pressed");
        pressed_keys.set_key(Key::Star);
        key_pressed = true;
    }

    // Set C2 High and read Row values for presses
    gpio_write(I2C_ADDR_KEYPAD, MASK_C2, i2c);
    let c2_presses = gpio_read(I2C_ADDR_KEYPAD, i2c);

    // Check for "2" press
    if c2_presses & MASK_R1 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '2' Pressed");
        pressed_keys.set_key(Key::Two);
        key_pressed = true;
    }
    // Check for "5" press
    if c2_presses & MASK_R2 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '5' Pressed");
        pressed_keys.set_key(Key::Five);
        key_pressed = true;
    }
    // Check for "8" press
    if c2_presses & MASK_R3 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '8' Pressed");
        pressed_keys.set_key(Key::Eight);
        key_pressed = true;
    }
    // Check for "0" press
    if c2_presses & MASK_R4 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '0' Pressed");
        pressed_keys.set_key(Key::Zero);
        key_pressed = true;
    }

    // Set C3 High and read Row values for presses
    gpio_write(I2C_ADDR_KEYPAD, MASK_C3, i2c);
    let c3_presses = gpio_read(I2C_ADDR_KEYPAD, i2c);

    // Check for "3" press
    if c3_presses & MASK_R1 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '3' Pressed");
        pressed_keys.set_key(Key::Three);
        key_pressed = true;
    }
    // Check for "6" press
    if c3_presses & MASK_R2 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '6' Pressed");
        pressed_keys.set_key(Key::Six);
        key_pressed = true;
    }
    // Check for "9" press
    if c3_presses & MASK_R3 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '9' Pressed");
        pressed_keys.set_key(Key::Nine);
        key_pressed = true;
    }
    // Check for "#" press
    if c3_presses & MASK_R4 > 0 {
        #[cfg(feature = "debug_keypad")]
        rprintln!("DEBUG_KEYPAD: '#' Pressed");
        pressed_keys.set_key(Key::Pound);
        key_pressed = true;
    }

    if key_pressed {
        Some(pressed_keys)
    } else {
        None
    }
}
