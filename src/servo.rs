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

use core::fmt::Debug;
use microbit::hal::{
    gpio::{Output, Pin, PushPull},
    pwm,
};

///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const FIFTY_HZ_IN_500KHZ_TICKS: u16 = 10000;
const PWM_ENABLE: u32 = 1;
const TRIGGER_TASK: u32 = 1;
const INFINITY_SAFE_LOOP_CNT: u32 = 2;

const DECODER_CMP_VALUE_MASK: u16 = 0x7FFF;

///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

pub struct Servo<T: pwm::Instance> {
    pwm_inst: T,
    _channel: pwm::Channel,
    _output_pin: Pin<Output<PushPull>>,
    common_duty: [u16; 2],
}

///////////////////////////////////////////////////////////////////////////////
//  Object Implementations
///////////////////////////////////////////////////////////////////////////////

impl<T: pwm::Instance> Servo<T> {
    pub fn new(pwm_inst: T, channel: pwm::Channel, output_pin: Pin<Output<PushPull>>) -> Self {
        // Before configuring, trigger TASKS_STOP to ensure a stable reset state
        pwm_inst
            .tasks_stop
            .write(|w| unsafe { w.bits(TRIGGER_TASK) });

        // Set the output pin
        pwm_inst.psel.out[0].write(|w| unsafe { w.bits(output_pin.psel_bits()) });

        // Enable the PWM Generator
        pwm_inst.enable.write(|w| unsafe { w.bits(PWM_ENABLE) });

        // Set the frequency to 50Hz via PRESCALER, COUNTERTOP register
        pwm_inst.prescaler.write(|w| w.prescaler().div_32()); // Divide clock down to 500kHz
        pwm_inst
            .countertop
            .write(|w| unsafe { w.countertop().bits(FIFTY_HZ_IN_500KHZ_TICKS) });

        // Set the duty cycle to 0 (common to all channels due to default DECODER config)
        let common_duty = [0, 0];
        pwm_inst
            .seq0
            .ptr
            .write(|w| unsafe { w.ptr().bits(common_duty.as_ptr() as u32) });
        pwm_inst.seq0.cnt.write(|w| unsafe { w.bits(1) });
        pwm_inst.seq0.refresh.write(|w| unsafe { w.bits(0) });

        // Configure PWM to loop infinitely on common duty cycle sequence by setting loop count > 1
        // and enabling shortcut that will link the LOOPSDONE even and SEQSTART[0]
        pwm_inst
            .loop_
            .write(|w| unsafe { w.bits(INFINITY_SAFE_LOOP_CNT) });
        pwm_inst.shorts.write(|w| w.loopsdone_seqstart0().enabled());

        // Note: The following registers are left untouched as their reset values are already the desired values:
        // MODE, DECODER, SEQ[0].ENDDELAY, SEQ[1].*

        Self {
            pwm_inst,
            _channel: channel,
            _output_pin: output_pin,
            common_duty,
        }
    }

    pub fn set_duty(&mut self, duty: f32) {
        // Set the new duty cycle
        self.common_duty = [
            ((FIFTY_HZ_IN_500KHZ_TICKS as f32 / 100.0) * duty) as u16 | 1 << 15,
            0,
        ];
        self.pwm_inst
            .seq0
            .ptr
            .write(|w| unsafe { w.ptr().bits(self.common_duty.as_ptr() as u32) });

        // Start the sequence again
        self.pwm_inst.tasks_seqstart[0].write(|w| unsafe { w.bits(TRIGGER_TASK) });
    }
}

impl<T: pwm::Instance> Debug for Servo<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Gather register values
        let shorts = self.pwm_inst.shorts.read().bits();
        let enable = self.pwm_inst.enable.read().bits();
        let mode = self.pwm_inst.mode.read().bits();
        let countertop = self.pwm_inst.countertop.read().bits();
        let prescaler = self.pwm_inst.prescaler.read().bits();
        let decoder = self.pwm_inst.decoder.read().bits();
        let loop_ = self.pwm_inst.loop_.read().bits();
        let seq0_ptr = self.pwm_inst.seq0.ptr.read().bits();
        let seq0_cnt = self.pwm_inst.seq0.cnt.read().bits();
        let seq1_ptr = self.pwm_inst.seq1.ptr.read().bits();
        let seq1_cnt = self.pwm_inst.seq1.cnt.read().bits();
        let psel = self.pwm_inst.psel.out[0].read().bits();

        // UNSAFELY retrieve seq values
        let seq0_val = unsafe { *(seq0_ptr as *const i32) };
        let seq1_val = unsafe { *(seq1_ptr as *const i32) };

        f.write_fmt(format_args!("Servo Register Map:\n"))?;
        f.write_fmt(format_args!("  SHORTS:         {:0>8b}\n", shorts))?;
        f.write_fmt(format_args!("  ENABLE:         {:0>8b}\n", enable))?;
        f.write_fmt(format_args!("  MODE:           {:0>8b}\n", mode))?;
        f.write_fmt(format_args!(
            "  COUNTERTOP:     {:0>8b} ({})\n",
            countertop, countertop
        ))?;
        f.write_fmt(format_args!(
            "  PRESCALER:      {:0>8b} ({})\n",
            prescaler, prescaler
        ))?;
        f.write_fmt(format_args!("  DECODER:        {:0>8b}\n", decoder))?;
        f.write_fmt(format_args!(
            "  LOOP:           {:0>8b} ({})\n",
            loop_, loop_
        ))?;
        f.write_fmt(format_args!(
            "  SEQ[0]PTR, CNT: {:0>8x}, {:0>8b} ({})\n",
            seq0_ptr, seq0_cnt, seq0_cnt
        ))?;
        f.write_fmt(format_args!(
            "  SEQ[0] Value:   {:0>16b} ({})\n",
            seq0_val,
            seq0_val & DECODER_CMP_VALUE_MASK as i32
        ))?;
        f.write_fmt(format_args!(
            "  SEQ[1]PTR, CNT: {:0>8x}, {:0>8b} ({})\n",
            seq1_ptr, seq1_cnt, seq1_cnt
        ))?;
        f.write_fmt(format_args!(
            "  SEQ[1] Value:   {:0>16b} ({})\n",
            seq1_val,
            seq1_val & DECODER_CMP_VALUE_MASK as i32
        ))?;
        f.write_fmt(format_args!("  PSEL.OUT[0]:    {:0>8b}\n", psel))?;

        Ok(())
    }
}
