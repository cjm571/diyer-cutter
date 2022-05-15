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

use core::{cell::RefCell, ops::DerefMut};

use cortex_m::interrupt::{free, Mutex};
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

static G_TIMER1: Mutex<RefCell<Option<Timer<TIMER1>>>> = Mutex::new(RefCell::new(None));


///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const ONE_SECOND_IN_MHZ: u32 = 1000000;


#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Take ownership of the full board
    let board = Board::take().unwrap();

    // Initialize a 1-second timer
    let mut timer1 = Timer::new(board.TIMER1);
    free(|cs| {
        // Stop and clear the timer before configuring it
        timer1.task_stop().write(|w| w.tasks_stop().trigger());
        timer1.task_clear().write(|w| w.tasks_clear().trigger());

        // [2022-05-15] Don't need to set prescaler or bit mode because microbit crate
        // currently hardcode these to 1MHz and 32bit mode

        // Enable and unmask the interrupt
        timer1.enable_interrupt();
        unsafe {
            NVIC::unmask(Interrupt::TIMER1);
        }

        // [2022-05-15] microbit crate currently does not expose the SHORTS register space
        // Cannot configure timer to auto-reset on reaching the specified tick count

        // Start the timer
        timer1.start(ONE_SECOND_IN_MHZ);

        G_TIMER1.borrow(cs).replace(Some(timer1));
    });

    // Instantiate a timer
    let mut timer = Timer::new(board.TIMER0);

    // Instantiate the LCD and initialize
    let input_pins = LcdInputPins::new(
        [
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
        ],
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

    free(|cs| {
        if let Some(ref mut timer1) = G_TIMER1.borrow(cs).borrow_mut().deref_mut() {
            // Clear the interrupt flag
            timer1.event_compare_cc0().write(|w| w.events_compare().not_generated());

            // Start another 1-second timer
            timer1.start(ONE_SECOND_IN_MHZ);
        }
    });
}
