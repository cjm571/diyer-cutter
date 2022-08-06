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
        gpio::{Input, Level, Pin, PullDown},
        pac::{twim0::frequency::FREQUENCY_A, Interrupt, NVIC, TWIM0},
        prelude::*,
        twim, Timer, Twim,
    },
    pac::{TIMER0, TIMER1},
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use rtic::app;

#[app(device = microbit::pac, peripherals = true)]
mod app {
    use super::*;

    ///////////////////////////////////////////////////////////////////////////////
    //  Named Constants
    ///////////////////////////////////////////////////////////////////////////////

    const ONE_SECOND_IN_MHZ: u32 = 1000000;
    const I2C_SLAVE_ADDR: u8 = 0b0100000;


    #[shared]
    struct Shared {
        timer0: Timer<TIMER0>,
        twim0: Twim<TWIM0>,
        i2c_verf_pins: [Pin<Input<PullDown>>; 8],
    }

    #[local]
    struct Local {
        timer1: Timer<TIMER1>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        // Take ownership of the full board
        let board = Board::new(cx.device, cx.core);

        // Instantiate a timer
        let timer0 = Timer::new(board.TIMER0);

        // Initialize a 1-second timer
        let mut timer1 = Timer::new(board.TIMER1);
        // Stop and clear the timer before configuring it
        timer1.task_stop().write(|w| w.tasks_stop().trigger());
        timer1.task_clear().write(|w| w.tasks_clear().trigger());

        // [2022-05-15] Don't need to set prescaler or bit mode because microbit crate
        // currently hardcodes these to 1MHz and 32bit mode

        // Enable and unmask the interrupt
        timer1.enable_interrupt();
        unsafe {
            NVIC::unmask(Interrupt::TIMER1);
        }

        // [2022-05-15] microbit crate currently does not expose the SHORTS register space
        // Cannot configure timer to auto-reset on reaching the specified tick count

        // Start the timer
        timer1.start(ONE_SECOND_IN_MHZ);

        // Create an instance of the TWIM0 (I2C) device.
        let twim0 = Twim::new(
            board.TWIM0,
            twim::Pins::from(board.i2c_external),
            FREQUENCY_A::K100,
        );

        // Create an array of GPIO pins for checking I2C chip
        let i2c_verf_pins: [Pin<Input<PullDown>>; 8] = [
            board.pins.p0_02.into_pulldown_input().degrade(), // P0
            board.pins.p0_03.into_pulldown_input().degrade(), // P1
            board.pins.p0_04.into_pulldown_input().degrade(), // P2
            board.display_pins.col3.into_pulldown_input().degrade(), // P3
            board.display_pins.col1.into_pulldown_input().degrade(), // P4
            board.display_pins.col4.into_pulldown_input().degrade(), // P6
            board.display_pins.col2.into_pulldown_input().degrade(), // P7
            board.pins.p0_10.into_pulldown_input().degrade(), // P8
        ];

        // Reset all I2C chips via I2C Reset Pin (P16)
        let mut i2c_reset_pin = board.pins.p1_02.into_push_pull_output(Level::High);
        i2c_reset_pin.set_low().unwrap();
        timer1.delay_us(1_u32);
        i2c_reset_pin.set_high().unwrap();

        (
            Shared {
                timer0,
                twim0,
                i2c_verf_pins,
            },
            Local { timer1 },
            init::Monotonics(),
        )
    }


    #[idle(shared = [timer0, twim0, &i2c_verf_pins])]
    fn idle(mut cx: idle::Context) -> ! {
        rprintln!("Entering main loop");

        // Write a dummy message out to the LCD I2C chip
        cx.shared.twim0.lock(|twim0| {
            let buffer = [0x00; 8];
            twim0.write(I2C_SLAVE_ADDR, &buffer).unwrap();
        });


        // let mut count = 0;
        loop {
            cx.shared.timer0.lock(|timer0| {
                timer0.delay_ms(100_u32);
            });

            rprintln!(
                "I2C Verf: 0b{}{}{}{}{}{}{}{}",
                cx.shared.i2c_verf_pins[7].is_high().unwrap() as u8,
                cx.shared.i2c_verf_pins[6].is_high().unwrap() as u8,
                cx.shared.i2c_verf_pins[5].is_high().unwrap() as u8,
                cx.shared.i2c_verf_pins[4].is_high().unwrap() as u8,
                cx.shared.i2c_verf_pins[3].is_high().unwrap() as u8,
                cx.shared.i2c_verf_pins[2].is_high().unwrap() as u8,
                cx.shared.i2c_verf_pins[1].is_high().unwrap() as u8,
                cx.shared.i2c_verf_pins[0].is_high().unwrap() as u8,
            );
        }
    }


    #[task(binds = TIMER1, shared = [&i2c_verf_pins], local = [timer1])]
    fn timer1(cx: timer1::Context) {
        // Clear the timer interrupt flag
        cx.local
            .timer1
            .event_compare_cc0()
            .write(|w| w.events_compare().not_generated());

        // Start another 1-second timer
        cx.local.timer1.start(ONE_SECOND_IN_MHZ);
    }
}
