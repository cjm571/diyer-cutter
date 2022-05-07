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

use microbit::{
    hal::{
        gpio::{Output, Pin, PushPull},
        pwm::{self, Pwm},
    },
    pac::PWM0,
};


///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

pub struct MotorDC {
    control_pin: Pin<Output<PushPull>>,
}


///////////////////////////////////////////////////////////////////////////////
//  Object Implementation
///////////////////////////////////////////////////////////////////////////////

impl MotorDC {
    pub fn new(control_pin: Pin<Output<PushPull>>) -> Self {
        Self { control_pin }
    }

    pub fn initialize(self, pwm0: PWM0) {
        // Set up PWM on micro:bit pin P16
        let pwm = Pwm::new(pwm0);
        pwm.set_output_pin(pwm::Channel::C0, self.control_pin);
    }
}
