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

///////////////////////////////////////////////////////////////////////////////
//  Module Declarations
///////////////////////////////////////////////////////////////////////////////

use microbit::{
    board::I2CExternalPins,
    hal::{
        gpio::{Output, Pin, PushPull},
        prelude::*,
        twim, Twim,
    },
    pac::twim0::frequency::FREQUENCY_A,
};

pub mod lcd1602;
pub mod keypad;

///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

pub const I2C_ADDR_LCD: u8 = 0b0100000;
pub const I2C_ADDR_KEYPAD: u8 = 0b0100001;

const GPIO_REG_ADDR: u8 = MCP23008Register::GPIO as u8;


///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum MCP23008Register {
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

/////////////////////////////////////////////////////////////////////////////
//  Helper Functions
///////////////////////////////////////////////////////////////////////////////

pub fn init<T: twim::Instance>(
    instance: T,
    i2c_pins: I2CExternalPins,
    reset_pin: &mut Pin<Output<PushPull>>,
) -> Twim<T> {
    // Create the TWIM object
    let i2c_device = Twim::new(instance, twim::Pins::from(i2c_pins), FREQUENCY_A::K100);

    // Pull all I2C devices out of reset
    reset_pin.set_high().unwrap();

    i2c_device
}

pub fn register_value_set<U: twim::Instance>(i2c_addr: u8, reg_addr: MCP23008Register, value: u8, i2c: &mut Twim<U>) {
    let reg_addr_and_data: [u8; 2] = [reg_addr as u8, value];
    i2c.write(i2c_addr, &reg_addr_and_data).unwrap();
}

pub fn gpio_write<U: twim::Instance>(i2c_addr: u8, value: u8, i2c: &mut Twim<U>) {
    register_value_set(i2c_addr, MCP23008Register::GPIO, value, i2c);
}

pub fn gpio_read<U: twim::Instance>(i2c_addr: u8, i2c: &mut Twim<U>) -> u8 {
    // Must declare this locally or the I2C driver will panic
    let gpio_reg_addr = GPIO_REG_ADDR;
    
    let mut rd_buffer: [u8; 1] = [0x00];
    i2c.write_then_read(i2c_addr, &[gpio_reg_addr], &mut rd_buffer)
        .unwrap();

    rd_buffer[0]
}

pub fn gpio_set_rmw<U: twim::Instance>(i2c_addr: u8, mask_val: u8, i2c: &mut Twim<U>) {
    // Must declare this locally or the I2C driver will panic
    let gpio_reg_addr = GPIO_REG_ADDR;

    // Read value currently in specified register
    let mut rd_buffer: [u8; 1] = [0x00];
    i2c.write_then_read(i2c_addr, &[gpio_reg_addr], &mut rd_buffer)
        .unwrap();

    // Modify the read value with mask
    let modified_data = rd_buffer[0] | mask_val;

    // Write the modified value back
    let reg_addr_and_data: [u8; 2] = [gpio_reg_addr, modified_data];
    i2c.write(i2c_addr, &reg_addr_and_data).unwrap();
}

pub fn gpio_unset_rmw<U: twim::Instance>(i2c_addr: u8, mask_val: u8, i2c: &mut Twim<U>) {
    // Must declare this locally or the I2C driver will panic
    let gpio_reg_addr = GPIO_REG_ADDR;

    // Read value currently in specified register
    let mut rd_buffer: [u8; 1] = [0x00];
    i2c.write_then_read(i2c_addr, &[gpio_reg_addr], &mut rd_buffer)
        .unwrap();

    // Modify the read value with mask
    let modified_data = rd_buffer[0] & !mask_val;

    // Write the modified value back
    let reg_addr_and_data: [u8; 2] = [gpio_reg_addr, modified_data];
    i2c.write(i2c_addr, &reg_addr_and_data).unwrap();
}
