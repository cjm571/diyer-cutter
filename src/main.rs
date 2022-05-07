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

#![no_main]
#![no_std]

use cortex_m_rt::entry;
use microbit::{
    board::Board,
    hal::{gpio::Level, prelude::*, Timer},
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

pub mod lcd1602;
use lcd1602::{Lcd1602, LcdInputPins};

pub mod motors;
use motors::MotorDC;


#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Take ownership of the full board
    let board = Board::take().unwrap();

    // Instantiate a timer
    let mut timer = Timer::new(board.TIMER0);

    // Instantiate the LCD and initialize
    let input_pins = LcdInputPins::new(
        board.pins.p0_02.into_push_pull_output(Level::Low).degrade(), // P0
        board.pins.p0_03.into_push_pull_output(Level::Low).degrade(), // P1
        board.pins.p0_04.into_push_pull_output(Level::Low).degrade(), // P2
        board
            .display_pins
            .col3
            .into_push_pull_output(Level::Low)
            .degrade(), // P3
        board
            .display_pins
            .col1
            .into_push_pull_output(Level::Low)
            .degrade(), // P4
        board.pins.p0_17.into_push_pull_output(Level::Low).degrade(), // P13
        board
            .display_pins
            .col4
            .into_push_pull_output(Level::Low)
            .degrade(), // P6
        board
            .display_pins
            .col2
            .into_push_pull_output(Level::Low)
            .degrade(), // P7
        board.pins.p0_09.into_push_pull_output(Level::Low).degrade(), // P9
        board
            .display_pins
            .col5
            .into_push_pull_output(Level::Low)
            .degrade(), // P10
        board.pins.p0_01.into_push_pull_output(Level::Low).degrade(), // P14
    );
    let mut lcd = Lcd1602::new(input_pins);
    lcd.initialize(&mut timer);

    // Greet the user
    lcd.display_greeting(&mut timer);

    // Initialize the DC motor
    let motor_dc = MotorDC::new(board.pins.p1_02.into_push_pull_output(Level::Low).degrade());
    motor_dc.initialize(board.PWM0);


    rprintln!("Entering main loop");
    loop {
        if board.buttons.button_a.is_low().unwrap() {
            rprintln!("BTN_A Pressed!");
        }
        if board.buttons.button_b.is_low().unwrap() {
            rprintln!("BTN_B Pressed!");
        }

        timer.delay_ms(10_u32);
    }
}
