//! Provides a bitfield abstraction for the core atomic types. This crate is `no_std` compatible
//! by default, and does not itself use any `unsafe` code.
//!
//! __Note__: On `stable` this crate assumes the presence of the following atomics
//! which may cause compilation to fail on certain platforms.
//! * `Atomic{U,I}32` and smaller
//! * `Atomic{U,I}size`
//! * `Atomic{U,I}64` on 64 bit platforms
//!
//! The `nightly` feature of this crate enables `target_has_atomic` and uses
//! that instead to detect which atomic types are available.
//! 
//! # Usage Example
//! ```
//! use core::sync::atomic::{AtomicU8, Ordering::Relaxed};
//! use atomic_bitfield::AtomicBitField as _;
//! 
//! let flags = AtomicU8::new(0b1000);
//! 
//! let prev_state = flags.set_bit(0, Relaxed);
//! assert_eq!(prev_state, false);
//! assert_eq!(flags.load(Relaxed), 0b1001);
//! 
//! let prev_state = flags.toggle_bit(3, Relaxed);
//! assert_eq!(prev_state, true);
//! assert_eq!(flags.load(Relaxed), 0b0001);
//! 
//! let prev_state = flags.swap_bit(0, false, Relaxed);
//! assert_eq!(prev_state, true);
//! assert_eq!(flags.load(Relaxed), 0b0000);
//! ```

#![cfg_attr(feature = "nightly", feature(cfg_target_has_atomic))]
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

use bit_field::BitField as _;
use core::{
	mem,
	sync::atomic::{self, Ordering},
};

/// Generic trait for manipulating bits atomically.
pub trait AtomicBitField: Sized {
	/// Returns the number of bits in this atomic type.
	#[inline]
	fn bit_len() -> usize {
		mem::size_of::<Self>() * 8
	}

	/// Atomically sets the bit to `new_val` at index `bit` (zero-indexed), returning the previous value.
	///
	/// ## Panics
	///
	/// This method will panic if the bit index is out of bounds of the bit field.
	#[inline]
	fn swap_bit(&self, bit: usize, new_val: bool, ord: Ordering) -> bool {
		assert!(bit < Self::bit_len());

		if new_val {
			self.set_bit(bit, ord)
		} else {
			self.reset_bit(bit, ord)
		}
	}

	/// Atomically retrieves the bit at index `bit` (zero-indexed).
	///
	/// ## Panics
	///
	/// This method will panic if the bit index is out of bounds of the bit field.
	fn get_bit(&self, bit: usize, ord: Ordering) -> bool;

	/// Atomically sets the bit to `true` at index `bit` (zero-indexed), returning the previous value.
	///
	/// ## Panics
	///
	/// This method will panic if the bit index is out of bounds of the bit field.
	fn set_bit(&self, bit: usize, ord: Ordering) -> bool;

	/// Atomically resets the bit to `false` at index `bit` (zero-indexed), returning the previous value.
	///
	/// ## Panics
	///
	/// This method will panic if the bit index is out of bounds of the bit field.
	fn reset_bit(&self, bit: usize, ord: Ordering) -> bool;

	/// Atomically toggles the bit (`0 -> 1`, `1 -> 0`) at index `bit` (zero-indexed), returning the previous value.
	///
	/// ## Panics
	///
	/// This method will panic if the bit index is out of bounds of the bit field.
	fn toggle_bit(&self, bit: usize, ord: Ordering) -> bool;
}

macro_rules! atomic_bitfield_impl_generate {
	($($atomic_t:ty),*) => ($(
		impl AtomicBitField for $atomic_t {
			#[inline]
			fn get_bit(&self, bit: usize, ord: Ordering) -> bool {
				assert!(bit < Self::bit_len());
				self.load(ord) & (1 << bit) != 0
			}

			#[inline]
			fn set_bit(&self, bit: usize, ord: Ordering) -> bool {
				assert!(bit < Self::bit_len());
				let prev = self.fetch_or(1 << bit, ord);
				prev.get_bit(bit)
			}

			#[inline]
			fn reset_bit(&self, bit: usize, ord: Ordering) -> bool {
				assert!(bit < Self::bit_len());
				let prev = self.fetch_and(!(1 << bit), ord);
				prev.get_bit(bit)
			}

			#[inline]
			fn toggle_bit(&self, bit: usize, ord: Ordering) -> bool {
				assert!(bit < Self::bit_len());
				let prev = self.fetch_xor(1 << bit, ord);
				prev.get_bit(bit)
			}
		}
	)*)
}

cfg_if::cfg_if! {
	if #[cfg(feature = "nightly")] {
		#[cfg(target_has_atomic = "8")]
		atomic_bitfield_impl_generate!(atomic::AtomicU8, atomic::AtomicI8);

		#[cfg(target_has_atomic = "16")]
		atomic_bitfield_impl_generate!(atomic::AtomicU16, atomic::AtomicI16);

		#[cfg(target_has_atomic = "32")]
		atomic_bitfield_impl_generate!(atomic::AtomicU32, atomic::AtomicI32);

		#[cfg(target_has_atomic = "64")]
		atomic_bitfield_impl_generate!(atomic::AtomicU64, atomic::AtomicI64);

		#[cfg(target_has_atomic = "ptr")]
		atomic_bitfield_impl_generate!(atomic::AtomicUsize, atomic::AtomicIsize);
	} else {
		use atomic::{AtomicU8, AtomicU16, AtomicU32, AtomicUsize, AtomicI8, AtomicI16, AtomicI32, AtomicIsize};
		atomic_bitfield_impl_generate!(AtomicU8, AtomicU16, AtomicU32, AtomicUsize, AtomicI8, AtomicI16, AtomicI32, AtomicIsize);

		#[cfg(target_pointer_width = "64")]
		atomic_bitfield_impl_generate!(atomic::AtomicU64, atomic::AtomicI64);
	}
}

#[cfg(test)]
mod test;
