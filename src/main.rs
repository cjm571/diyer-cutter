#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use microbit::{
    board::{Board, Pins},
    gpio::DisplayPins,
    hal::{
        gpio::{p0, p1, Level, Output, PushPull},
        prelude::*,
        Timer,
    },
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

struct OutputPinsToLcd {
    d0: p0::P0_02<Output<PushPull>>,  // P0
    d1: p0::P0_03<Output<PushPull>>,  // P1
    d2: p0::P0_04<Output<PushPull>>,  // P2
    d3: p0::P0_31<Output<PushPull>>,  // P3
    d4: p0::P0_28<Output<PushPull>>,  // P4
    d5: p0::P0_17<Output<PushPull>>,  // P13
    d6: p1::P1_05<Output<PushPull>>,  // P6
    d7: p0::P0_11<Output<PushPull>>,  // P7
    rs: p0::P0_09<Output<PushPull>>,  // P9
    rw: p0::P0_30<Output<PushPull>>,  // P10
    en: p0::P0_01<Output<PushPull>>,  // P14
}


#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();

    // Set up pins as outputs and name according to their attachment on the LCD PCB
    let mut output_pins_to_lcd = OutputPinsToLcd::new(board.pins, board.display_pins);
    output_pins_to_lcd.print_state();

    let mut timer = Timer::new(board.TIMER0);

    // Wait a little bit before entering main loop
    rprintln!("Giving LCD time to initialize...");
    timer.delay_ms(1000_u32);

    // Set up LCD for 8-bit, 2-line mode
    output_pins_to_lcd.set_8bit_2line_mode();
    rprintln!("Setting LCD up for 8bit, 2line mode...");
    output_pins_to_lcd.print_state();

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    output_pins_to_lcd.reset_pins();
    timer.delay_ms(500_u32);
    output_pins_to_lcd.print_state();

    // Set up LCD cursor
    output_pins_to_lcd.set_cursor();
    rprintln!("Setting LCD cursor up...");
    output_pins_to_lcd.print_state();

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    output_pins_to_lcd.reset_pins();
    timer.delay_ms(500_u32);
    output_pins_to_lcd.print_state();

    // Set up auto-increment
    output_pins_to_lcd.set_autoincrement();
    rprintln!("Setting up auto-increment...");
    output_pins_to_lcd.print_state();

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    output_pins_to_lcd.reset_pins();
    timer.delay_ms(500_u32);
    output_pins_to_lcd.print_state();

    // Clear the display before writing anything
    output_pins_to_lcd.clear_display();
    rprintln!("Clearing display...");
    output_pins_to_lcd.print_state();    

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    output_pins_to_lcd.reset_pins();
    timer.delay_ms(500_u32);
    output_pins_to_lcd.print_state();

    // Write 'H'
    output_pins_to_lcd.write_H();
    rprintln!("Writing 'H'...");
    output_pins_to_lcd.print_state();

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    output_pins_to_lcd.reset_pins();
    timer.delay_ms(500_u32);
    output_pins_to_lcd.print_state();

    // Write 'I'
    output_pins_to_lcd.write_I();
    rprintln!("Writing 'I'...");
    output_pins_to_lcd.print_state();

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    output_pins_to_lcd.reset_pins();
    timer.delay_ms(500_u32);

    // Write 'I'
    output_pins_to_lcd.write_I();
    rprintln!("Writing 'I'...");
    output_pins_to_lcd.print_state();

    // Give LCD time to process and pull E back down TODO: Check BF
    timer.delay_ms(500_u32);
    output_pins_to_lcd.reset_pins();
    timer.delay_ms(500_u32);
    output_pins_to_lcd.print_state();

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

impl OutputPinsToLcd {
    pub fn new(pins: Pins, display_pins: DisplayPins) -> Self {
        Self {
            d0: pins.p0_02.into_push_pull_output(Level::Low),        // P0
            d1: pins.p0_03.into_push_pull_output(Level::Low),        // P1
            d2: pins.p0_04.into_push_pull_output(Level::Low),        // P2
            d3: display_pins.col3.into_push_pull_output(Level::Low), // P3 TODO: use a lower-level pin struct so we don't have the split-brain pins/display pins thing
            d4: display_pins.col1.into_push_pull_output(Level::Low), // P4
            d5: pins.p0_17.into_push_pull_output(Level::Low),        // P13
            d6: display_pins.col4.into_push_pull_output(Level::Low), // P6
            d7: display_pins.col2.into_push_pull_output(Level::Low), // P7
            rs: pins.p0_09.into_push_pull_output(Level::Low),        // P9
            rw: display_pins.col5.into_push_pull_output(Level::Low), // P10
            en: pins.p0_01.into_push_pull_output(Level::Low),        // P14
        }
    }

    pub fn reset_pins(&mut self) {
        self.en.set_low().unwrap();

        self.d0.set_low().unwrap();
        self.d1.set_low().unwrap();
        self.d2.set_low().unwrap();
        self.d3.set_low().unwrap();
        self.d4.set_low().unwrap();
        self.d5.set_low().unwrap();
        self.d6.set_low().unwrap();
        self.d7.set_low().unwrap();
        self.rs.set_low().unwrap();
        self.rw.set_low().unwrap();
    }

    pub fn clear_display(&mut self) {
        self.reset_pins();

        self.d0.set_high().unwrap();

        self.en.set_high().unwrap();
    }

    pub fn set_8bit_2line_mode(&mut self) {
        self.reset_pins();

        self.d5.set_high().unwrap();
        self.d4.set_high().unwrap();
        self.d3.set_high().unwrap();

        self.en.set_high().unwrap();
    }

    pub fn set_cursor(&mut self) {
        self.reset_pins();

        self.d3.set_high().unwrap();
        self.d2.set_high().unwrap();
        self.d1.set_high().unwrap();

        self.en.set_high().unwrap();
    }

    pub fn set_autoincrement(&mut self) {
        self.reset_pins();

        self.d2.set_high().unwrap();
        self.d1.set_high().unwrap();

        self.en.set_high().unwrap();
    }

    pub fn write_H(&mut self) {
        self.reset_pins();

        self.rs.set_high().unwrap();
        self.d6.set_high().unwrap();
        self.d3.set_high().unwrap();

        self.en.set_high().unwrap();
    }

    pub fn write_I(&mut self) {
        self.reset_pins();

        self.rs.set_high().unwrap();
        self.d6.set_high().unwrap();
        self.d3.set_high().unwrap();
        self.d0.set_high().unwrap();

        self.en.set_high().unwrap();
    }

    pub fn print_state(&self) {
        rprintln!(
            "rs rw d7 d6 d5 d4 d3 d2 d1 d0 en\n{}  {}  {}  {}  {}  {}  {}  {}  {}  {}  {}",
            self.rs.is_set_high().unwrap() as u32,
            self.rw.is_set_high().unwrap() as u32,
            self.d7.is_set_high().unwrap() as u32,
            self.d6.is_set_high().unwrap() as u32,
            self.d5.is_set_high().unwrap() as u32,
            self.d4.is_set_high().unwrap() as u32,
            self.d3.is_set_high().unwrap() as u32,
            self.d2.is_set_high().unwrap() as u32,
            self.d1.is_set_high().unwrap() as u32,
            self.d0.is_set_high().unwrap() as u32,
            self.en.is_set_high().unwrap() as u32,
        );
    }
}
