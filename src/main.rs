/* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *\
Copyright (C) 2023 CJ McAllister
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

use core::cell::RefCell;

use cortex_m::{
    interrupt::{self as cortex_interrupt, Mutex},
    peripheral::NVIC,
};

use microbit::{
    hal::{gpio::Level, prelude::*, timer, twim, Timer, Twim},
    pac::{interrupt, Interrupt, PWM0, TIMER0, TIMER1, TWIM0},
    Board,
};

mod i2c;
use crate::i2c::{
    keypad::{self, Key},
    lcd1602,
};

mod servo;
use servo::Servo;

///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const ONE_SECOND_IN_MHZ: u32 = 1000000;
const GREETING_DUR_IN_MS: u32 = 2500;

// Cap input to 5 digits for ease of implementation
const MAX_INPUT_CHARS: usize = 5;

const CUT_CYCLE_TIME_MS: u32 = 1500;
const WIRE_FEED_TIME_MS: u32 = 3000;

///////////////////////////////////////////////////////////////////////////////
//  Shared Peripheral Handles
///////////////////////////////////////////////////////////////////////////////

static TIMER0_HANDLE: Mutex<RefCell<Option<Timer<TIMER0>>>> = Mutex::new(RefCell::new(None));
static TIMER1_HANDLE: Mutex<RefCell<Option<Timer<TIMER1>>>> = Mutex::new(RefCell::new(None));
static I2C0_HANDLE: Mutex<RefCell<Option<Twim<TWIM0>>>> = Mutex::new(RefCell::new(None));
static CUTTER_HANDLE: Mutex<RefCell<Option<Servo<PWM0>>>> = Mutex::new(RefCell::new(None));

///////////////////////////////////////////////////////////////////////////////
//  Tasks
///////////////////////////////////////////////////////////////////////////////

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    init();

    defmt::println!("Initialization Complete!");

    idle();
}

fn init() {
    // Take ownership of the full board
    let board = Board::take().unwrap();

    // Hold various chips in reset/output-disabled
    let i2c_reset_pin = board.pins.p1_02.into_push_pull_output(Level::Low); // P16
    let mut lcd_lvshift_oe_pin = board.pins.p0_12.into_push_pull_output(Level::High); // P12

    // Instantiate a timer
    let timer0 = init_1s_timer(board.TIMER0);

    // Initialize a 1-second timer
    let timer1 = init_1s_timer(board.TIMER1);
    cortex_interrupt::free(|cs| TIMER1_HANDLE.borrow(cs).replace(Some(timer1)));

    // Initialize the TWIM0 (I2C) controller
    let mut i2c0 = i2c::init(
        board.TWIM0,
        board.i2c_external,
        &mut i2c_reset_pin.degrade(),
    );

    // Initialize LCD Display and display greeting
    defmt::println!("Enabling power to LCD Display...");
    lcd1602::power_on(&mut i2c0);

    defmt::println!("Enabling output on LCD Level Shifter...");
    lcd_lvshift_oe_pin.set_low().unwrap();

    defmt::println!("Initializing LCD Display...");
    cortex_interrupt::free(|cs| {
        let mut local_timer1_handle_ref = TIMER1_HANDLE.borrow(cs).borrow_mut();
        let local_timer1_handle = local_timer1_handle_ref.as_mut().unwrap();
        lcd1602::init(local_timer1_handle, &mut i2c0);
    });

    defmt::println!("Initializing 3x4 Matrix Keypad...");
    keypad::init(&mut i2c0);

    defmt::println!("Initializing Cutter Servo...");
    let pwm_output_pin = board.pins.p0_09.into_push_pull_output(Level::Low).degrade();
    let cutter = Servo::new(board.PWM0, microbit::hal::pwm::Channel::C0, pwm_output_pin);

    // Store the peripheral handles in RefCells, so interrupts and main thread can use them
    cortex_interrupt::free(|cs| TIMER0_HANDLE.borrow(cs).replace(Some(timer0)));
    cortex_interrupt::free(|cs| I2C0_HANDLE.borrow(cs).replace(Some(i2c0)));
    cortex_interrupt::free(|cs| CUTTER_HANDLE.borrow(cs).replace(Some(cutter)));
}

fn idle() -> ! {
    cortex_interrupt::free(|cs| {
        // Capture shared peripheral handles locally
        let mut local_timer0_handle_ref = TIMER0_HANDLE.borrow(cs).borrow_mut();
        let timer0 = local_timer0_handle_ref.as_mut().unwrap();
        let mut local_i2c0_handle_ref = I2C0_HANDLE.borrow(cs).borrow_mut();
        let i2c0 = local_i2c0_handle_ref.as_mut().unwrap();
        let mut local_cutter_handle_ref = CUTTER_HANDLE.borrow(cs).borrow_mut();
        let cutter = local_cutter_handle_ref.as_mut().unwrap();

        // Display greeting
        lcd1602::display_greeting(timer0, i2c0);
        timer0.delay_ms(GREETING_DUR_IN_MS);

        // Input Loop
        let mut cut_length;
        let mut num_cuts;
        loop {
            // Prompt user for Cut Length
            defmt::println!("Prompting user for Cut Length...");
            cut_length = get_user_parameter("CUT LENGTH (in):\n-> ", timer0, i2c0);
            defmt::println!("User accepted Cut Length of {}", cut_length);

            // Prompt user for Number of Cuts
            defmt::println!("Prompting user for Number of Cuts...");
            num_cuts = get_user_parameter("NUMBER OF CUTS:\n-> ", timer0, i2c0);
            defmt::println!("User accepted Number of Cuts of {}", num_cuts);

            // Present final confirmation
            defmt::println!("Presenting final confirmation to user...");
            if !final_confirmation(cut_length, num_cuts, timer0, i2c0) {
                // User rejected confirmation, return to top of input loop
                defmt::println!("User rejected confirmation");
                continue;
            } else {
                // User accepted confirmation, break out of input loop
                defmt::println!("User accepted confirmation");
                break;
            }
        }

        // Cutting Loop
        lcd1602::clear_display(timer0, i2c0);
        lcd1602::write_string("Cutting...\n00000 / ", timer0, i2c0);
        lcd1602::write_u32(num_cuts, timer0, i2c0);
        lcd1602::shift_cursor(lcd1602::Direction::Left, 8, timer0, i2c0);
        for i in 1..=num_cuts {
            // Update LCD
            lcd1602::backspace(5, timer0, i2c0);
            lcd1602::write_u32(i, timer0, i2c0);

            // Perform a single cut
            cutter.set_duty(12.0);
            timer0.delay_ms(CUT_CYCLE_TIME_MS);
            cutter.set_duty(3.0);

            // Allow time for wire feed
            timer0.delay_ms(WIRE_FEED_TIME_MS);
        }

        lcd1602::clear_display(timer0, i2c0);
        lcd1602::write_string("Finished Cutting\nWoohoo! <3", timer0, i2c0);
    });

    defmt::println!("Entering Idle loop");
    loop {
        cortex_interrupt::free(|cs| {
            // Capture shared peripheral handles locally
            let mut local_timer0_handle_ref = TIMER0_HANDLE.borrow(cs).borrow_mut();
            let timer0 = local_timer0_handle_ref.as_mut().unwrap();
            timer0.delay_ms(500_u32);
        });
    }
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
                for (order_of_magnitude, parsed_value) in user_input
                    .iter()
                    .rev()
                    .flat_map(|v| v.parse::<u32>())
                    .enumerate()
                {
                    parsed_input += parsed_value * u32::pow(10, order_of_magnitude as u32);
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

    loop {
        if let Some(pressed_key) = keypad::scan(timer, i2c) {
            if pressed_key == Key::Pound {
                // User accepted confirmation
                return true;
            } else if pressed_key == Key::Star {
                // User rejected confirmation
                return false;
            } else {
                continue;
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
//  Interrupt Handlers
///////////////////////////////////////////////////////////////////////////////

#[interrupt]
fn TIMER1() {
    // Clear the timer interrupt flag && restart 1s timer
    cortex_interrupt::free(|cs| {
        let mut local_timer1_handle_ref = TIMER1_HANDLE.borrow(cs).borrow_mut();
        let local_timer1_handle = local_timer1_handle_ref.as_mut().unwrap();
        local_timer1_handle
            .event_compare_cc0()
            .write(|w| w.events_compare().not_generated());

        local_timer1_handle.start(ONE_SECOND_IN_MHZ);
    });
}

///////////////////////////////////////////////////////////////////////////////
//  Embedded Boilerplate
///////////////////////////////////////////////////////////////////////////////

use cortex_m_semihosting::debug;

use defmt_rtt as _; // global logger

use microbit as _; // memory layout

use panic_probe as _;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes a semihosting-capable debug tool exit
/// with status code 0.
pub fn exit() -> ! {
    loop {
        debug::exit(debug::EXIT_SUCCESS);
    }
}

/// Hardfault handler.
///
/// Terminates the application and makes a semihosting-capable debug tool exit
/// with an error. This seems better than the default, which is to spin in a
/// loop.
#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE);
    }
}

// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert;

    #[test]
    fn it_works() {
        assert!(true)
    }
}
