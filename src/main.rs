#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;
use microbit::{
    board::Board,
    hal::{prelude::*, Timer, gpio::{Output, PushPull}},
};

struct LcdPinout {
    d0: microbit::hal::gpio::p0::P0_02<Output<PushPull>>,
    d1: microbit::hal::gpio::p0::P0_03<Output<PushPull>>,
    d2: microbit::hal::gpio::p0::P0_04<Output<PushPull>>,
    d3: microbit::hal::gpio::p0::P0_31<Output<PushPull>>,
    d4: microbit::hal::gpio::p0::P0_28<Output<PushPull>>,
    d5: microbit::hal::gpio::p0::P0_17<Output<PushPull>>,
    d6: microbit::hal::gpio::p1::P1_05<Output<PushPull>>,
    d7: microbit::hal::gpio::p0::P0_11<Output<PushPull>>,
    rs: microbit::hal::gpio::p0::P0_09<Output<PushPull>>,
    rw: microbit::hal::gpio::p0::P0_30<Output<PushPull>>,
    e: microbit::hal::gpio::p0::P0_01<Output<PushPull>>,
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);

    // Set up pins as outputs and name according to their attachment on the LCD PCB
    let mut lcd_pinout = LcdPinout {
        d0: board.pins.p0_02.into_push_pull_output(microbit::hal::gpio::Level::Low), // P0
        d1: board.pins.p0_03.into_push_pull_output(microbit::hal::gpio::Level::Low), // P1
        d2: board.pins.p0_04.into_push_pull_output(microbit::hal::gpio::Level::Low), // P2
        d3: board.display_pins.col3.into_push_pull_output(microbit::hal::gpio::Level::Low), // P3
        d4: board.display_pins.col1.into_push_pull_output(microbit::hal::gpio::Level::Low), // P4
        d5: board.pins.p0_17.into_push_pull_output(microbit::hal::gpio::Level::Low), // P13
        d6: board.display_pins.col4.into_push_pull_output(microbit::hal::gpio::Level::Low), // P6
        d7: board.display_pins.col2.into_push_pull_output(microbit::hal::gpio::Level::Low), // P7
        rs: board.pins.p0_09.into_push_pull_output(microbit::hal::gpio::Level::Low), // P9
        rw: board.display_pins.col5.into_push_pull_output(microbit::hal::gpio::Level::Low), // P10
        e: board.pins.p0_01.into_push_pull_output(microbit::hal::gpio::Level::Low), // P14
    };

    // Wait a little bit before entering main loop
    rprintln!("Giving LCD time to initialize...");
    timer.delay_ms(1000_u32);

    // Set up LCD for 8-bit, 2-line mode
    lcd_pinout.d5.set_high();
    lcd_pinout.d4.set_high();
    lcd_pinout.d3.set_high();
    lcd_pinout.e.set_high();
    rprintln!("Setting LCD up for 8bit, 2line mode...");

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    lcd_pinout.e.set_low();
    timer.delay_ms(500_u32);

    // Set up LCD cursor
    lcd_pinout.d5.set_low();
    lcd_pinout.d4.set_low();
    lcd_pinout.d2.set_high();
    lcd_pinout.d1.set_high();
    lcd_pinout.e.set_high();
    rprintln!("Setting LCD cursor up...");

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    lcd_pinout.e.set_low();
    timer.delay_ms(500_u32);

    // Set up auto-increment
    lcd_pinout.d3.set_low();
    lcd_pinout.e.set_high();
    rprintln!("Setting up auto-increment...");

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    lcd_pinout.e.set_low();
    timer.delay_ms(500_u32);

    // Write 'H'
    lcd_pinout.reset_pins();
    lcd_pinout.rs.set_high();
    lcd_pinout.d6.set_high();
    lcd_pinout.d3.set_high();
    lcd_pinout.e.set_high();
    rprintln!("Writing 'H'...");

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    lcd_pinout.e.set_low();
    timer.delay_ms(500_u32);

    // Write 'I'
    lcd_pinout.reset_pins();
    lcd_pinout.rs.set_high();
    lcd_pinout.d6.set_high();
    lcd_pinout.d3.set_high();
    lcd_pinout.d0.set_high(); 
    lcd_pinout.e.set_high();
    rprintln!("Writing 'I'...");

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    lcd_pinout.e.set_low();
    timer.delay_ms(500_u32);

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

impl LcdPinout {

    pub fn reset_pins(&mut self) -> &Self {
        self.d0.set_low();
        self.d1.set_low();
        self.d2.set_low();
        self.d3.set_low();
        self.d4.set_low();
        self.d5.set_low();
        self.d6.set_low();
        self.d7.set_low();
        self.rs.set_low();
        self.rw.set_low();
        self.e.set_low();

        self
    }
}