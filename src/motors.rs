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

use microbit::hal::{
    gpio::{Output, Pin, PushPull},
    prelude::*,
    pwm::{self, Pwm},
};


///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

pub struct MotorDC<'p, T>
where
    T: pwm::Instance,
{
    output_pin: Pin<Output<PushPull>>,
    pwm: &'p Pwm<T>,
}


///////////////////////////////////////////////////////////////////////////////
//  Object Implementation
///////////////////////////////////////////////////////////////////////////////

impl<'p, T> MotorDC<'p, T>
where
    T: pwm::Instance,
{
    pub fn new(output_pin: Pin<Output<PushPull>>, pwm: &'p Pwm<T>) -> Self {
        Self { output_pin, pwm }
    }

    pub fn initialize(self) {
        // Set up PWM on micro:bit pin P16
        self.pwm
            .set_output_pin(pwm::Channel::C0, self.output_pin)
            .set_period(500_u32.hz())
            .set_duty_on_common(self.pwm.get_max_duty());
    }
}
