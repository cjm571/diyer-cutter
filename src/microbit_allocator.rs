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

use core::{cell::UnsafeCell, sync::atomic::{AtomicUsize, Ordering}, alloc::{GlobalAlloc, Layout}, ptr::null_mut};

extern crate alloc;


///////////////////////////////////////////////////////////////////////////////
//  Named Constants
///////////////////////////////////////////////////////////////////////////////

const ARENA_SIZE: usize = 16* 1024; // Using half of the available RAM per memory.x
const MAX_SUPPORTED_ALIGN: usize = 4096;


///////////////////////////////////////////////////////////////////////////////
//  Data Structures
///////////////////////////////////////////////////////////////////////////////

#[repr(C, align(4096))] // 4096 == MAX_SUPPORTED_ALIGN
struct MicrobitAllocator {
    arena: UnsafeCell<[u8; ARENA_SIZE]>,
    remaining: AtomicUsize, // we allocate from the top, counting down
}

#[global_allocator]
static ALLOCATOR: MicrobitAllocator = MicrobitAllocator {
    arena: UnsafeCell::new([0x55; ARENA_SIZE]),
    remaining: AtomicUsize::new(ARENA_SIZE),
};


///////////////////////////////////////////////////////////////////////////////
//  Object Implementation
///////////////////////////////////////////////////////////////////////////////

unsafe impl Sync for MicrobitAllocator {}

unsafe impl GlobalAlloc for MicrobitAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // `Layout` contract forbids making a `Layout` with align=0, or align not power of 2.
        // So we can safely use a mask to ensure alignment without worrying about UB.
        let align_mask_to_round_down = !(align - 1);

        if align > MAX_SUPPORTED_ALIGN {
            return null_mut();
        }

        let mut allocated = 0;
        if self
            .remaining
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |mut remaining| {
                if size > remaining {
                    return None;
                }
                remaining -= size;
                remaining &= align_mask_to_round_down;
                allocated = remaining;
                Some(remaining)
            })
            .is_err()
        {
            return null_mut();
        };
        (self.arena.get() as *mut u8).add(allocated)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Do nothing! :D
    }
}