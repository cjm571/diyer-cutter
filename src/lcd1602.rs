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
        Timer,
    },
    pac::TIMER0,
};
use rtt_target::rprintln;

///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const LCD_MAX_LINE_LENGTH: usize = 16;
const LCD_MAX_NEWLINES: usize = 1;


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

    pub fn initialize(&mut self, timer: &mut Timer<TIMER0>) {
        // Wait a little bit before entering main loop
        rprintln!("Giving LCD time to initialize...");
        timer.delay_ms(15_u32);

        // Set up LCD for 8-bit, 2-line mode
        self.input_pins.set_8bit_2line_mode(timer);
        rprintln!("Setting LCD up for 8bit, 2line mode...");

        // Set up LCD cursor
        self.input_pins.set_cursor(timer);
        rprintln!("Setting LCD cursor up...");

        // Set up auto-increment
        self.input_pins.set_autoincrement(timer);
        rprintln!("Setting up auto-increment...");

        // Clear the display before anything is written
        self.input_pins.clear_display(timer);
        rprintln!("Clearing display...");
    }

    pub fn display_greeting(&mut self, timer: &mut Timer<TIMER0>) {
        // Write "HI BABE!"
        self.input_pins
            .write_string("HI BABE! <3\nYou so pretty...", timer);
        rprintln!("Writing greeting...");
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

    fn pulse_enable(&mut self, timer: &mut Timer<TIMER0>) {
        // Delay before setting Enable high to ensure that Address Settling time (60ns) is not violated
        timer.delay_us(1000_u32);
        self.en.set_high().unwrap();

        // Enable must be held high for at least 500ns (at 3.3V operation) per HD44780U datasheet
        // Shortest we can hold is 1us, so we'll do that
        timer.delay_us(1000_u32);

        self.en.set_low().unwrap();

        // Delay another 1us to ensure Enable Cycle Time minimum (1000ns) is not violated
        timer.delay_us(1000_u32);
    }

    pub fn reset_pins(&mut self) {
        for i in 0..=7 {
            self.d[i].set_low().unwrap();
        }
        self.rs.set_low().unwrap();
        self.rw.set_low().unwrap();
    }

    pub fn clear_display(&mut self, timer: &mut Timer<TIMER0>) {
        self.reset_pins();

        self.d[0].set_high().unwrap();

        self.pulse_enable(timer);
    }

    pub fn set_8bit_2line_mode(&mut self, timer: &mut Timer<TIMER0>) {
        self.reset_pins();

        self.d[5].set_high().unwrap();
        self.d[4].set_high().unwrap();
        self.d[3].set_high().unwrap();

        self.pulse_enable(timer);
    }

    pub fn set_cursor(&mut self, timer: &mut Timer<TIMER0>) {
        self.reset_pins();

        self.d[3].set_high().unwrap();
        self.d[2].set_high().unwrap();
        self.d[1].set_high().unwrap();

        self.pulse_enable(timer);
    }

    pub fn set_autoincrement(&mut self, timer: &mut Timer<TIMER0>) {
        self.reset_pins();

        self.d[2].set_high().unwrap();
        self.d[1].set_high().unwrap();

        self.pulse_enable(timer);
    }

    pub fn write_string(&mut self, out_str: &str, timer: &mut Timer<TIMER0>) {
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
                self.newline(timer);
            } else {
                self.write_char(c, timer);
            }
        }
    }

    pub fn backspace(&mut self, timer: &mut Timer<TIMER0>) {
        // Shift cursor backwards
        self.shift_cursor(Direction::Left, timer);

        // Write a blank character code
        self.write_char(32 as char, timer);

        // Shift cursor backwards again in prep for next char entry
        self.shift_cursor(Direction::Left, timer);
    }

    fn write_char(&mut self, c: char, timer: &mut Timer<TIMER0>) {
        self.reset_pins();
        self.rs.set_high().unwrap();

        // Get the ASCII index of the character
        let ascii_idx = c as u32;

        // Check each bit's value and set in the corresponding data bit pin
        for i in 0..=7 {
            if ascii_idx & (1 << i) != 0 {
                self.d[i].set_high().unwrap();
            }
        }

        self.pulse_enable(timer);
    }

    fn shift_cursor(&mut self, dir: Direction, timer: &mut Timer<TIMER0>) {
        self.reset_pins();

        self.d[4].set_high().unwrap();

        // Left == low, Right == high
        if dir == Direction::Right {
            self.d[2].set_high().unwrap();
        }

        self.pulse_enable(timer);
    }

    fn newline(&mut self, timer: &mut Timer<TIMER0>) {
        self.reset_pins();

        self.d[7].set_high().unwrap();
        self.d[6].set_high().unwrap();

        self.pulse_enable(timer);
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
