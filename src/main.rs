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
    board::{Board, I2CExternalPins},
    hal::{
        gpio::{Input, Level, Output, Pin, PullDown, PushPull},
        pac::{twim0::frequency::FREQUENCY_A, Interrupt, NVIC, TWIM0},
        prelude::*,
        timer, twim, Timer, Twim,
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

    /* MCP23008 Consts */
    const I2C_SLAVE_ADDR: u8 = 0b0100000;


    ///////////////////////////////////////////////////////////////////////////////
    //  Data Structures
    ///////////////////////////////////////////////////////////////////////////////

    #[allow(dead_code)]
    enum MCP23008Register {
        IODIR = 0x00,
        IPOL = 0x01,
        GPINTEN = 0x02,
        DEFVEL = 0x03,
        INTCON = 0x04,
        IOCON = 0x05,
        GPPU = 0x06,
        INTF = 0x07,
        INTCAP = 0x08,
        GPIO = 0x09,
        OLAT = 0x0A,
    }

    #[shared]
    struct Shared {
        timer0: Timer<TIMER0>,
        i2c0: Twim<TWIM0>,
        i2c_verf_pins: [Pin<Input<PullDown>>; 8],
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

        // Instantiate a timer
        let timer0 = Timer::new(board.TIMER0);

        // Initialize a 1-second timer
        let mut timer1 = init_1s_timer(board.TIMER1);

        // Initialize the TWIM0 (I2C) device
        let i2c_reset_pin = board.pins.p1_02.into_push_pull_output(Level::High);
        let i2c0 = init_i2c(
            board.TWIM0,
            board.i2c_external,
            &mut i2c_reset_pin.degrade(),
            &mut timer1,
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

        (
            Shared {
                timer0,
                i2c0,
                i2c_verf_pins,
            },
            Local { timer1 },
            init::Monotonics(),
        )
    }


    #[idle(shared = [timer0, i2c0, &i2c_verf_pins])]
    fn idle(mut cx: idle::Context) -> ! {
        rprintln!("Entering main loop");

        rprintln!(
            "Initial I2C Verf: 0b{}{}{}{}{}{}{}{}",
            cx.shared.i2c_verf_pins[7].is_high().unwrap() as u8,
            cx.shared.i2c_verf_pins[6].is_high().unwrap() as u8,
            cx.shared.i2c_verf_pins[5].is_high().unwrap() as u8,
            cx.shared.i2c_verf_pins[4].is_high().unwrap() as u8,
            cx.shared.i2c_verf_pins[3].is_high().unwrap() as u8,
            cx.shared.i2c_verf_pins[2].is_high().unwrap() as u8,
            cx.shared.i2c_verf_pins[1].is_high().unwrap() as u8,
            cx.shared.i2c_verf_pins[0].is_high().unwrap() as u8,
        );

        // Write some MCP23008 Registers to verify I2C functionality
        cx.shared.i2c0.lock(|i2c0| {
            let reg_addr: [u8; 1] = [MCP23008Register::GPIO as u8];
            let mut rd_buffer: [u8; 1] = [0x00];
            i2c0.write_then_read(I2C_SLAVE_ADDR, &reg_addr, &mut rd_buffer)
                .unwrap();
            rprintln!("GPIO: {:0>8b}", rd_buffer[0]);

            let reg_addr_and_wr_buffer: [u8; 2] = [MCP23008Register::GPIO as u8, 0b10101010];
            i2c0.write(I2C_SLAVE_ADDR, &reg_addr_and_wr_buffer).unwrap();

            rd_buffer = [0x00];
            i2c0.write_then_read(I2C_SLAVE_ADDR, &reg_addr, &mut rd_buffer)
                .unwrap();
            rprintln!("GPIO: {:0>8b}", rd_buffer[0]);
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

    fn init_i2c<T: twim::Instance, U: timer::Instance>(
        instance: T,
        i2c_pins: I2CExternalPins,
        reset_pin: &mut Pin<Output<PushPull>>,
        timer_device: &mut Timer<U>,
    ) -> Twim<T> {
        // Create the TWIM object
        let mut i2c_device = Twim::new(instance, twim::Pins::from(i2c_pins), FREQUENCY_A::K100);

        // Reset all I2C chips via I2C Reset Pin
        reset_pin.set_low().unwrap();
        timer_device.delay_us(1_u32);
        reset_pin.set_high().unwrap();

        // Set all pins on LCD Display's MCP23008 to Output mode
        let reg_addr_and_wr_buffer: [u8; 2] = [MCP23008Register::IODIR as u8, 0b00000000];
        i2c_device
            .write(I2C_SLAVE_ADDR, &reg_addr_and_wr_buffer)
            .unwrap();

        i2c_device
    }
}
