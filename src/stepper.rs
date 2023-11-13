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

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayUs;
use microbit::hal::{gpio::{PushPull, Output, Pin}, timer, prelude::{OutputPin, StatefulOutputPin}, Timer};


///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const MIN_STEP_CYCLE_TIME_US: u32 = 66;


///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

pub struct Stepper {
    step_pin: Pin<Output<PushPull>>,
    dir_pin: Pin<Output<PushPull>>,
}


///////////////////////////////////////////////////////////////////////////////
//  Object Implementations
///////////////////////////////////////////////////////////////////////////////
 
impl Stepper {
    pub fn new(mut step_pin: Pin<Output<PushPull>>, mut dir_pin: Pin<Output<PushPull>>) -> Self {
        // Put the pins into a known state (both LOW)
        step_pin.set_low().unwrap();
        dir_pin.set_low().unwrap();
        
        Self {
            step_pin,
            dir_pin,
        }
    }

    pub fn step_forward<T: timer::Instance>(&mut self, ticks: u32, timer: &mut Timer<T>) {
        // Check state of the DIR pin before setting
        if self.dir_pin.is_set_low().unwrap() {
            self.dir_pin.set_low().unwrap();
        }

        // Pulse the STEP pin the specified number of times
        for _ in 0..ticks {
            self.step_pin.set_high().unwrap();
            timer.delay_us(MIN_STEP_CYCLE_TIME_US);
            self.step_pin.set_low().unwrap();
            timer.delay_us(MIN_STEP_CYCLE_TIME_US);
        }
    }
}
