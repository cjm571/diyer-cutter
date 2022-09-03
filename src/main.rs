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

use microbit::{
    board::Board,
    hal::{
        gpio::Level,
        pac::{Interrupt, NVIC, TWIM0},
        prelude::*,
        timer, Timer, Twim,
    },
    pac::{TIMER0, TIMER1},
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use rtic::app;

mod i2c_periphs;
use crate::i2c_periphs::{I2C_ADDR_KEYPAD, init_i2c, rmw_mask_val_set, rmw_mask_val_unset};
use crate::i2c_periphs::lcd1602;


#[app(device = microbit::pac, peripherals = true)]
mod app {
    use super::*;

    ///////////////////////////////////////////////////////////////////////////////
    //  Named Constants
    ///////////////////////////////////////////////////////////////////////////////

    const ONE_SECOND_IN_MHZ: u32 = 1000000;


    ///////////////////////////////////////////////////////////////////////////////
    //  Data Structures
    ///////////////////////////////////////////////////////////////////////////////

    #[shared]
    struct Shared {
        timer0: Timer<TIMER0>,
        i2c0: Twim<TWIM0>,
    }

    #[local]
    struct Local {
        timer1: Timer<TIMER1>,
    }


    ///////////////////////////////////////////////////////////////////////////////
    //  RTIC Tasks
    ///////////////////////////////////////////////////////////////////////////////

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        // Take ownership of the full board
        let board = Board::new(cx.device, cx.core);

        // Hold various chips in reset/output-disabled
        let i2c_reset_pin = board.pins.p1_02.into_push_pull_output(Level::Low); // P16
        let mut lcd_lvshift_oe_pin = board.pins.p0_12.into_push_pull_output(Level::High); // P12

        // Instantiate a timer
        let timer0 = init_1s_timer(board.TIMER0);

        // Initialize a 1-second timer
        let mut timer1 = init_1s_timer(board.TIMER1);

        // Initialize the TWIM0 (I2C) device
        let mut i2c0 = init_i2c(
            board.TWIM0,
            board.i2c_external,
            &mut i2c_reset_pin.degrade(),
        );

        // Initialize LCD Display and display greeting
        rprintln!("Enabling power to LCD Display...");
        lcd1602::power_on(&mut i2c0);

        rprintln!("Enabling output on LCD Level Shifter...");
        lcd_lvshift_oe_pin.set_low().unwrap();

        rprintln!("Initializing LCD Display...");
        lcd1602::initialize(&mut timer1, &mut i2c0);
        lcd1602::display_greeting(&mut timer1, &mut i2c0);

        (
            Shared { timer0, i2c0 },
            Local { timer1 },
            init::Monotonics(),
        )
    }


    #[idle(shared = [timer0, i2c0])]
    fn idle(mut cx: idle::Context) -> ! {
        rprintln!("Entering main loop");

        let mut all_on = false;
        loop {
            cx.shared.timer0.lock(|timer0| {
                timer0.delay_ms(500_u32);
            });

            cx.shared.i2c0.lock(|i2c0| {
                if all_on {
                    rmw_mask_val_unset(I2C_ADDR_KEYPAD, 0b11111111, i2c0);
                    all_on = false;
                }
                else {
                    rmw_mask_val_set(I2C_ADDR_KEYPAD, 0b11111111, i2c0);
                    all_on = true;
                }
            })
        }
    }


    #[task(binds = TIMER1, local = [timer1])]
    fn timer1(cx: timer1::Context) {
        // Clear the timer interrupt flag
        cx.local
            .timer1
            .event_compare_cc0()
            .write(|w| w.events_compare().not_generated());

        // Start another 1-second timer
        cx.local.timer1.start(ONE_SECOND_IN_MHZ);
    }


    ///////////////////////////////////////////////////////////////////////////////
    //  Helper Functions
    ///////////////////////////////////////////////////////////////////////////////

    fn init_1s_timer<T: timer::Instance>(instance: T) -> Timer<T> {
        // Create the Timer object
        let mut timer_device = Timer::new(instance);

        // Stop and clear the timer before configuring it
        timer_device.task_stop().write(|w| w.tasks_stop().trigger());
        timer_device
            .task_clear()
            .write(|w| w.tasks_clear().trigger());

        // [2022-05-15] Don't need to set prescaler or bit mode because microbit crate
        // currently hardcodes these to 1MHz and 32bit mode

        // Enable and unmask the interrupt
        timer_device.enable_interrupt();
        unsafe {
            NVIC::unmask(Interrupt::TIMER1);
        }

        // [2022-05-15] microbit crate currently does not expose the SHORTS register space
        // Cannot configure timer to auto-reset on reaching the specified tick count

        // Start the timer
        timer_device.start(ONE_SECOND_IN_MHZ);

        timer_device
    }
}
