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
    hal::{
        gpio::Level,
        pac::{interrupt, Interrupt, NVIC},
        prelude::*,
        pwm::Pwm,
        Timer,
    },
    pac::TIMER1,
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

pub mod lcd1602;
use lcd1602::{Lcd1602, LcdInputPins};

pub mod motors;
use motors::MotorDC;

///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const ONE_SECOND_IN_MHZ: u32 = 1000000;


#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Initialize a 1-second timer
    unsafe {
        let timer1 = TIMER1::ptr();
        // Stop and clear the timer before configuring it
        (*timer1).tasks_stop.write(|w| w.tasks_stop().trigger());
        (*timer1).tasks_clear.write(|w| w.tasks_clear().trigger());

        // Set prescaler to 4, which will give us a 1MHz clock (16MHz / 2^4)
        (*timer1).prescaler.write(|w| w.prescaler().bits(4));

        // Set 32bit mode so we can fully check against a 1-second timeout in MHz
        (*timer1).bitmode.write(|w| w.bitmode()._32bit());

        // Set 1-second comparison value in the Capture/Compare register 0
        (*timer1).cc[0].write(|w| w.cc().bits(ONE_SECOND_IN_MHZ));

        // Enable and unmask the interrupt
        (*timer1).intenset.write(|w| w.compare0().set());
        NVIC::unmask(Interrupt::TIMER1);

        // Set up the shortcut that will reset the timer when it reaches 1-second
        (*timer1).shorts.write(|w| w.compare0_clear().enabled());

        // Start the timer and u
        (*timer1).tasks_start.write(|w| w.tasks_start().trigger());
    }

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

    // Initialize the "DC motor"
    let pwm0 = Pwm::new(board.PWM0);
    let motor_dc = MotorDC::new(
        board.pins.p1_02.into_push_pull_output(Level::Low).degrade(),
        &pwm0,
    );
    motor_dc.initialize();

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


#[interrupt]
fn TIMER1() {
    rprintln!("TIMER1 INTERRUPT!");

    // Reset the interrupt flag
    let timer1 = TIMER1::ptr();
    unsafe {
        (*timer1).events_compare[0].write(|w| w.events_compare().not_generated());
    }
}
