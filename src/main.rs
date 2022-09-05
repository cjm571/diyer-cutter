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
        timer, twim, Timer, Twim,
    },
    pac::{TIMER0, TIMER1},
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use rtic::app;

mod i2c;
use crate::i2c::{
    keypad::{self, Key},
    lcd1602,
};


#[app(device = microbit::pac, peripherals = true)]
mod app {
    use super::*;

    ///////////////////////////////////////////////////////////////////////////////
    //  Named Constants
    ///////////////////////////////////////////////////////////////////////////////

    const ONE_SECOND_IN_MHZ: u32 = 1000000;
    const GREETING_DUR_IN_MS: u32 = 2500;

    // Cap input to 5 digits for ease of implementation
    const MAX_INPUT_CHARS: usize = 5;


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

        // Initialize the TWIM0 (I2C) controller
        let mut i2c0 = i2c::init(
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
        lcd1602::init(&mut timer1, &mut i2c0);

        rprintln!("Initializing 3x4 Matrix Keypad...");
        keypad::init(&mut i2c0);

        (
            Shared { timer0, i2c0 },
            Local { timer1 },
            init::Monotonics(),
        )
    }


    #[idle(shared = [timer0, i2c0])]
    fn idle(mut cx: idle::Context) -> ! {
        (&mut cx.shared.timer0, &mut cx.shared.i2c0).lock(|timer, i2c| {
            // Display greeting
            lcd1602::display_greeting(timer, i2c);
            timer.delay_ms(GREETING_DUR_IN_MS);

            loop {
                // Prompt user for Cut Length
                rprintln!("Prompting user for Cut Length...");
                let cut_length = get_user_parameter("CUT LENGTH (in):\n-> ", timer, i2c);
                rprintln!("User accepted Cut Length of {}", cut_length);

                // Prompt user for Number of Cuts
                rprintln!("Prompting user for Number of Cuts...");
                let num_cuts = get_user_parameter("NUMBER OF CUTS:\n-> ", timer, i2c);
                rprintln!("User accepted Number of Cuts of {}", num_cuts);

                // Present final confirmation
                rprintln!("Presenting final confirmation to user...");
                if !final_confirmation(cut_length, num_cuts, timer, i2c) {
                    // User rejected confirmation, return to top of input loop
                    rprintln!("User rejected confirmation");
                    continue;
                } else {
                    // User confirmed, break out of input loop
                    rprintln!("User accepted confirmation");
                    lcd1602::clear_display(timer, i2c);
                    lcd1602::write_string("Input accepted!\nCutting...", timer, i2c);
                    break;
                }
            }
        });

        rprintln!("Entering Idle loop");
        loop {
            (&mut cx.shared.timer0).lock(|timer| {
                timer.delay_ms(500_u32);
            });
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

    fn get_user_parameter<T: timer::Instance, U: twim::Instance>(
        prompt: &str,
        timer: &mut Timer<T>,
        i2c: &mut Twim<U>,
    ) -> u32 {
        lcd1602::clear_display(timer, i2c);
        lcd1602::write_string(prompt, timer, i2c);

        let mut user_input: [&str; 12] = [""; 12];
        let mut user_input_idx = 0;
        loop {
            if let Some(pressed_key) = keypad::scan(timer, i2c) {
                // Check for '#', which will parse and accept the input
                if pressed_key == Key::Pound {
                    let mut parsed_input = 0;
                    let mut order_of_magnitude = 0;
                    for parsed_value in user_input.iter().rev().map(|v| v.parse::<u32>()) {
                        if let Ok(value) = parsed_value {
                            parsed_input += value * u32::pow(10, order_of_magnitude);
                            order_of_magnitude += 1;
                        }
                    }

                    return parsed_input;
                }
                // Check for '*', which acts as a backspace key
                if pressed_key == Key::Star {
                    //OPT: Beep if input is empty?
                    // Don't allow backspace if input is empty
                    if user_input_idx > 0 {
                        lcd1602::backspace(1, timer, i2c);
                        user_input_idx -= 1;
                        user_input[user_input_idx] = "";
                    }

                    continue;
                }

                //OPT: Beep if input is full?
                // If not at max length, write the key to the LCD and record it in the input array
                if user_input_idx < MAX_INPUT_CHARS {
                    lcd1602::write_string(pressed_key.into(), timer, i2c);
                    user_input[user_input_idx] = pressed_key.into();
                    user_input_idx += 1;
                }
            } else {
                continue;
            }
        }
    }

    fn final_confirmation<T: timer::Instance, U: twim::Instance>(
        cut_length: u32,
        num_cuts: u32,
        timer: &mut Timer<T>,
        i2c: &mut Twim<U>,
    ) -> bool {
        lcd1602::clear_display(timer, i2c);
        lcd1602::write_u32(cut_length, timer, i2c);
        lcd1602::write_string("in x ", timer, i2c);
        lcd1602::write_u32(num_cuts, timer, i2c);
        lcd1602::write_string("\nOK? (#=Y, *=N) ", timer, i2c);

        //FIXME: DEBUG DELETE
        timer.delay_ms(2500_u32);
        return true;
    }
}
