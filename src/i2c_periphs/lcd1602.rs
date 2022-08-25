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
        gpio::{Output, Pin, PushPull, Input, PullDown},
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

const MASK_RS: u8 = 0b00000001;
const MASK_RW: u8 = 0b00000010;
const MASK_EN: u8 = 0b00000100;
const MASK_D4: u8 = 0b00001000;
const MASK_D5: u8 = 0b00010000;
const MASK_D6: u8 = 0b00100000;
const MASK_D7: u8 = 0b01000000;
const MASK_ALL: u8 = 0b01111111;
const MASK_NONE: u8 = 0b00000000;


///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

pub struct Lcd1602 {
    input_pins: LcdInputPins,
}

pub struct LcdInputPins {
    i2c_verf_pins: [Pin<Input<PullDown>>; 8]
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
        // Allow time for LCD VCC to rist to 4.5V
        rprintln!("Giving LCD time to initialize...");
        timer.delay_ms(1000_u32);

        // // Manually reset the LCD controller
        // rprintln!("Manually resetting LCD via instruction sequence...");
        // self.manual_reset(timer, i2c);

        // Set up LCD for 8-bit, 2-line mode
        self.input_pins.set_4bit_2line_mode(timer, i2c);
        rprintln!("Setting LCD up for 4bit, 2line mode...");

        // Set up LCD cursor
        self.input_pins.set_cursor(timer, i2c);
        rprintln!("Setting LCD cursor up...");

        // Set up auto-increment
        self.input_pins.set_autoincrement(timer, i2c);
        rprintln!("Setting up auto-increment...");

        // Clear the display before anything is written
        self.input_pins.clear_display(timer, i2c);
        rprintln!("Clearing display...");

        //FIXME: DEBUG DELETE
        rprintln!("LCD Initilization Complete");
    }

    pub fn initialize_4b_1l<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // 1. Allow time for LCD VCC to rist to 4.5V
        rprintln!("Giving LCD time to initialize...");
        timer.delay_ms(1000_u32);

        // 2. Set up LCD for 4-bit operation
        rprintln!("Setting LCD up for 4bit Operation...");
        self.input_pins.set_4bit_op(timer, i2c);

        // 3. Set 4-bit operation (again) and selects 1-line display
        rprintln!("Setting LCD up for 1-Line Mode...");
        self.input_pins.set_4bit_op(timer, i2c);
        self.input_pins.set_1line_mode(timer, i2c);

        // // 3. Set 4-bit operation (again) and selects 1-line display
        // rprintln!("Manually Resetting...");
        // self.input_pins.manual_reset(timer, i2c);

        // 4. Turn on display/cursor
        rprintln!("Turning on LCD Display and Cursor...");
        self.input_pins.set_cursor(timer, i2c);

        // 5. Entry mode set
        rprintln!("Setting entry mode to INCR, no SHIFT...");
        self.input_pins.set_autoincrement(timer, i2c);


        //FIXME: DEBUG DELETE
        rprintln!("LCD Initialization Complete");
    }

    //TODO: Fix this weird straddled architecture
    pub fn manual_reset<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.input_pins.manual_reset(timer, i2c);
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
    pub fn new(i2c_verf_pins: [Pin<Input<PullDown>>; 8]) -> Self {
        Self {i2c_verf_pins}
    }

    pub fn manual_reset<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // 1. Function Set pt. 1
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D4 | MASK_D5, i2c);
        self.pulse_enable(timer, i2c);

        // 2. Wait >4.1ms
        timer.delay_ms(1000_u32);

        // 3. Function Set pt. 2 (Same values as pt 1)
        self.pulse_enable(timer, i2c);

        // 4. Wait for >100us
        timer.delay_ms(1000_u32);

        // 5. Function Set pt. 3 (Same values as pt 1, again)
        self.pulse_enable(timer, i2c);

        // Wait for > execution time
        timer.delay_ms(1000_u32);

        // 6. Set interface data length to 4 bits
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D5, i2c);
        self.pulse_enable(timer, i2c);

        // 7. (Final) Function Set, higher-order bits are same as last command
        self.pulse_enable(timer, i2c);
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D7, i2c);
        self.pulse_enable(timer, i2c);
        
        // Wait for > execution time
        timer.delay_ms(1000_u32);

        // 8. Turn display off
        self.reset_pins(timer, i2c);
        self.pulse_enable(timer, i2c);
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D7, i2c);
        self.pulse_enable(timer, i2c);
        
        // Wait for > execution time
        timer.delay_ms(1000_u32);

        // 9. Clear display
        self.reset_pins(timer, i2c);
        self.pulse_enable(timer, i2c);
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D4, i2c);
        self.pulse_enable(timer, i2c);
        
        // Wait for > execution time
        timer.delay_ms(1000_u32);

        // 10. Entry mode set
        self.reset_pins(timer, i2c);
        self.pulse_enable(timer, i2c);
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D5 | MASK_D6, i2c);
        self.pulse_enable(timer, i2c);

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
        #[cfg(feature = "debug_pulse_en")]
        {
            rprintln!("+++ EN PULSE");
            rprintln!("_7654SWE");
            rprintln!(
                "{}{}{}{}{}{}{}{}",
                self.i2c_verf_pins[0].is_high().unwrap() as u8,
                self.i2c_verf_pins[1].is_high().unwrap() as u8,
                self.i2c_verf_pins[2].is_high().unwrap() as u8,
                self.i2c_verf_pins[3].is_high().unwrap() as u8,
                self.i2c_verf_pins[4].is_high().unwrap() as u8,
                self.i2c_verf_pins[5].is_high().unwrap() as u8,
                self.i2c_verf_pins[6].is_high().unwrap() as u8,
                self.i2c_verf_pins[7].is_high().unwrap() as u8,
            );
        }
        
        // Delay before setting EN high to ensure that Address Set-Up time (tAS, 40ms) is not violated
        timer.delay_us(40_u32);

        // Set EN high
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_EN, i2c);
        #[cfg(feature = "debug_pulse_en")]
        {
            rprintln!(
                "{}{}{}{}{}{}{}{}",
                self.i2c_verf_pins[0].is_high().unwrap() as u8,
                self.i2c_verf_pins[1].is_high().unwrap() as u8,
                self.i2c_verf_pins[2].is_high().unwrap() as u8,
                self.i2c_verf_pins[3].is_high().unwrap() as u8,
                self.i2c_verf_pins[4].is_high().unwrap() as u8,
                self.i2c_verf_pins[5].is_high().unwrap() as u8,
                self.i2c_verf_pins[6].is_high().unwrap() as u8,
                self.i2c_verf_pins[7].is_high().unwrap() as u8,
            );
        }

        // Hold EN high for the required time from the datasheet (PWEH, 230ms)
        timer.delay_us(230_u32);

        // Set EN low
        rmw_mask_val_unset(MCP23008Register::GPIO, MASK_EN, timer, i2c);
        #[cfg(feature = "debug_pulse_en")]
        {
            rprintln!(
                "{}{}{}{}{}{}{}{}\n",
                self.i2c_verf_pins[0].is_high().unwrap() as u8,
                self.i2c_verf_pins[1].is_high().unwrap() as u8,
                self.i2c_verf_pins[2].is_high().unwrap() as u8,
                self.i2c_verf_pins[3].is_high().unwrap() as u8,
                self.i2c_verf_pins[4].is_high().unwrap() as u8,
                self.i2c_verf_pins[5].is_high().unwrap() as u8,
                self.i2c_verf_pins[6].is_high().unwrap() as u8,
                self.i2c_verf_pins[7].is_high().unwrap() as u8,
            );
        }

        // Delay before allowing other operations (remainder of tcycE, 270ms)
        timer.delay_us(270_u32);
    }

    pub fn reset_pins<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        #[cfg(feature = "debug_reset")]
        {
            rprintln!("--- PRE-RESET");
            rprintln!("_7654SWE");
            rprintln!(
                "{}{}{}{}{}{}{}{}",
                self.i2c_verf_pins[0].is_high().unwrap() as u8,
                self.i2c_verf_pins[1].is_high().unwrap() as u8,
                self.i2c_verf_pins[2].is_high().unwrap() as u8,
                self.i2c_verf_pins[3].is_high().unwrap() as u8,
                self.i2c_verf_pins[4].is_high().unwrap() as u8,
                self.i2c_verf_pins[5].is_high().unwrap() as u8,
                self.i2c_verf_pins[6].is_high().unwrap() as u8,
                self.i2c_verf_pins[7].is_high().unwrap() as u8,
            );
        }
        
        rmw_mask_val_unset(MCP23008Register::GPIO, MASK_ALL, timer, i2c);

        #[cfg(feature = "debug_reset")]
        {
            rprintln!("--- POST-RESET");
            rprintln!("_7654SWE");
            rprintln!(
                "{}{}{}{}{}{}{}{}\n",
                self.i2c_verf_pins[0].is_high().unwrap() as u8,
                self.i2c_verf_pins[1].is_high().unwrap() as u8,
                self.i2c_verf_pins[2].is_high().unwrap() as u8,
                self.i2c_verf_pins[3].is_high().unwrap() as u8,
                self.i2c_verf_pins[4].is_high().unwrap() as u8,
                self.i2c_verf_pins[5].is_high().unwrap() as u8,
                self.i2c_verf_pins[6].is_high().unwrap() as u8,
                self.i2c_verf_pins[7].is_high().unwrap() as u8,
            );
        }
    }

    pub fn clear_display<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // Higher-order data bits write
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_NONE, i2c);
        self.pulse_enable(timer, i2c);

        // Lower-order data bits write
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D4, i2c);
        self.pulse_enable(timer, i2c);
    }

    pub fn set_4bit_op<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D5, i2c);
        self.pulse_enable(timer, i2c);    
    }

    pub fn set_1line_mode<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.reset_pins(timer, i2c);
        self.pulse_enable(timer, i2c);    
    }

    pub fn set_4bit_2line_mode<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // First phase of Function Set command - sets 4-bit operation mode (just one write, unlike most others)
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D5, i2c);
        self.pulse_enable(timer, i2c);

        // Second phase of Function Set command - sets 4-bit, 2-line mode
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D5, i2c);
        self.pulse_enable(timer, i2c);
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D7, i2c);
        self.pulse_enable(timer, i2c);
    }

    pub fn set_cursor<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // Higher-order data bits write
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_NONE, i2c);
        self.pulse_enable(timer, i2c);

        // Lower-order data bits write
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D5 | MASK_D6 | MASK_D7, i2c);
        self.pulse_enable(timer, i2c);
    }

    pub fn set_autoincrement<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // Higher-order data bits write
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_NONE, i2c);
        self.pulse_enable(timer, i2c);

        // Lower-order data bits write
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D5 | MASK_D6, i2c);
        self.pulse_enable(timer, i2c);
    }

    fn write_char<T: timer::Instance, U: twim::Instance>(&mut self, c: char, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_RS, i2c);

        // Get the ASCII index of the character
        let ascii_idx = c as u32;

        // Check each higher-order bit's value and set in the corresponding data bit pin
        //TODO: This can very likely be optimized by using the shifted value directly
        if ascii_idx & (1 << 4) != 0 {
            rmw_mask_val_set(MCP23008Register::GPIO, MASK_D4, i2c);
        }
        if ascii_idx & (1 << 5) != 0 {
            rmw_mask_val_set(MCP23008Register::GPIO, MASK_D5, i2c);
        }
        if ascii_idx & (1 << 6) != 0 {
            rmw_mask_val_set(MCP23008Register::GPIO, MASK_D6, i2c);
        }
        if ascii_idx & (1 << 7) != 0 {
            rmw_mask_val_set(MCP23008Register::GPIO, MASK_D7, i2c);
        }
        self.pulse_enable(timer, i2c);

        // Check each lower-order bit's value and set in the corresponding data bit pin
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_RS, i2c);
        //TODO: This can very likely be optimized by using the shifted value directly
        if ascii_idx & (1 << 0) != 0 {
            rmw_mask_val_set(MCP23008Register::GPIO, MASK_D4, i2c);
        }
        if ascii_idx & (1 << 1) != 0 {
            rmw_mask_val_set(MCP23008Register::GPIO, MASK_D5, i2c);
        }
        if ascii_idx & (1 << 2) != 0 {
            rmw_mask_val_set(MCP23008Register::GPIO, MASK_D6, i2c);
        }
        if ascii_idx & (1 << 3) != 0 {
            rmw_mask_val_set(MCP23008Register::GPIO, MASK_D7, i2c);
        }
        self.pulse_enable(timer, i2c);



    }

    fn shift_cursor<T: timer::Instance, U: twim::Instance>(&mut self, dir: Direction, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // Higher-order data bits write
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D4, i2c);
        self.pulse_enable(timer, i2c);

        // Lower-order data bits write
        self.reset_pins(timer, i2c);
        // Left == low, Right == high
        if dir == Direction::Right {
            rmw_mask_val_set(MCP23008Register::GPIO, MASK_D6, i2c);
        }
        self.pulse_enable(timer, i2c);
    }

    fn newline<T: timer::Instance, U: twim::Instance>(&mut self, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
        // Higher-order data bits write
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_D6 | MASK_D7, i2c);
        self.pulse_enable(timer, i2c);

        // Lower-order data bits write
        self.reset_pins(timer, i2c);
        rmw_mask_val_set(MCP23008Register::GPIO, MASK_NONE, i2c);
        self.pulse_enable(timer, i2c);
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

fn rmw_mask_val_unset<T: timer::Instance, U: twim::Instance>(reg_addr: MCP23008Register, mask_val: u8, timer: &mut Timer<T>, i2c: &mut Twim<U>) {
    // Read value current in specified register
    let mut rd_buffer: [u8;1] = [0x00];
    i2c.write_then_read(I2C_ADDR_LCD, &[reg_addr as u8], &mut rd_buffer).unwrap();

    // Modify the read value with mask
    let modified_data = rd_buffer[0] & !mask_val;

    // Write the modified value back
    let reg_addr_and_data: [u8; 2] = [reg_addr as u8, modified_data]; 
    i2c.write(I2C_ADDR_LCD, &reg_addr_and_data).unwrap();
}