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
        pac::{Interrupt, NVIC},
        prelude::*,
        Timer,
    },
    pac::{TIMER0, TIMER1},
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use rtic::app;

#[app(device = microbit::pac, peripherals = true, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    ///////////////////////////////////////////////////////////////////////////////
    //  Named Constants
    ///////////////////////////////////////////////////////////////////////////////

    const ONE_SECOND_IN_MHZ: u32 = 1000000;


    #[shared]
    struct Shared {
        timer0: Timer<TIMER0>,
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

        (Shared { timer0 }, Local { timer1 }, init::Monotonics())
    }


    #[idle(shared = [timer0])]
    fn idle(mut cx: idle::Context) -> ! {
        rprintln!("Entering main loop");

        let mut count = 0;
        loop {
            cx.shared.timer0.lock(|timer0| {
                timer0.delay_ms(100_u32);
            });

            count += 1;
            rprintln!("Loop #{}", count);
        }
    }


    #[task(binds = TIMER1, local = [timer1])]
    fn timer1(cx: timer1::Context) {
        static mut COUNTER: u8 = 0;
        rprintln!("TIMER1 INTERRUPT!");

        // Clear the timer interrupt flag
        cx.local
            .timer1
            .event_compare_cc0()
            .write(|w| w.events_compare().not_generated());
        // });

        // Start another 1-second timer
        cx.local.timer1.start(ONE_SECOND_IN_MHZ);

        #[allow(unused_unsafe)]
        unsafe {
            COUNTER += 1;
        }
    }
}
