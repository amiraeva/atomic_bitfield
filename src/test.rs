use quickcheck::quickcheck;

use super::AtomicBitField;

use std::{
	mem::size_of,
	sync::atomic::{Ordering::Relaxed, *},
};

#[test]
fn test_bit_len() {
	assert_eq!(AtomicU8::bit_len(), 8);
	assert_eq!(AtomicU16::bit_len(), 16);
	assert_eq!(AtomicU32::bit_len(), 32);
	assert_eq!(AtomicUsize::bit_len(), 8 * size_of::<usize>());

	assert_eq!(AtomicI8::bit_len(), 8);
	assert_eq!(AtomicI16::bit_len(), 16);
	assert_eq!(AtomicI32::bit_len(), 32);
	assert_eq!(AtomicIsize::bit_len(), 8 * size_of::<isize>());
}

fn to_atomic_and_clamp<A, Int>((int, bit): &(Int, u8)) -> (A, u8)
where
	A: AtomicBitField + From<Int>,
	Int: Copy,
{
	(A::from(*int), bit & (A::bit_len() - 1) as u8)
}

// sets a bit, swaps it back, outputs if the result is the same as the original
fn bit_flipping<A, Int>(test_vals: Vec<(Int, u8)>) -> bool
where
	A: AtomicBitField + AtomicLoad<Inner = Int> + From<Int>,
	Int: Copy + PartialEq,
{
	let ints = test_vals.iter().map(|(int, _)| *int);

	let set_bit = |(val, bit): (A, u8)| {
		let prev = val.set_bit(bit as _, Relaxed);
		(val, bit, prev)
	};

	let swap_bit = |(val, bit, prev): (A, u8, bool)| {
		val.swap_bit(bit as _, prev, Relaxed);
		val
	};

	test_vals
		.iter()
		.map(to_atomic_and_clamp)
		.map(set_bit)
		.map(swap_bit)
		.map(|a| a.load())
		.eq(ints)
}

// toggles a bit, toggles it back, outputs if the result is the same as the original
fn bit_toggling<A, Int>(test_vals: Vec<(Int, u8)>) -> bool
where
	A: AtomicBitField + AtomicLoad<Inner = Int> + From<Int>,
	Int: Copy + PartialEq,
{
	let ints = test_vals.iter().map(|(int, _)| *int);

	let to_int = |(val, _): (A, _)| val.load();

	let toggle = |(val, bit): &(A, u8)| {
		val.toggle_bit(*bit as _, Relaxed);
	};

	test_vals
		.iter()
		.map(to_atomic_and_clamp)
		.inspect(toggle)
		.inspect(toggle)
		.map(to_int)
		.eq(ints)
}

macro_rules! bit_manipulation_test_impl {
	($flip:ident, $toggle:ident; $($atomic_t:ty, $primitive_t:ident);*) => (
		mod $flip {
			use super::*;
			$(
				quickcheck! {
					fn $primitive_t(test_vals: Vec<($primitive_t, u8)>) -> bool {
						bit_flipping::<$atomic_t, _>(test_vals)
					}
				}
			)*
		}

		mod $toggle {
			use super::*;
			$(
				quickcheck! {
					fn $primitive_t(test_vals: Vec<($primitive_t, u8)>) -> bool {
						bit_toggling::<$atomic_t, _>(test_vals)
					}
				}
			)*
		}
	)
}

bit_manipulation_test_impl![
	bit_flip, bit_toggle;

	AtomicU8, u8;
	AtomicU16, u16;
	AtomicU32, u32;
	AtomicUsize, usize;

	AtomicI8, i8;
	AtomicI16, i16;
	AtomicI32, i32;
	AtomicIsize, isize
];

#[cfg(target_pointer_width = "64")]
bit_manipulation_test_impl![
	bit_flip_64, bit_toggle_64;

	AtomicU64, u64;
	AtomicI64, i64
];

trait AtomicLoad {
	type Inner;
	fn load(&self) -> Self::Inner;
}

macro_rules! atomic_load_impl {
	($($atomic_t:ty, $primitive_t:ty);*) => ($(
		impl AtomicLoad for $atomic_t {
			type Inner = $primitive_t;
			fn load(&self) -> Self::Inner { self.load(Relaxed) }
		}
	)*)
}

atomic_load_impl!(
	AtomicU8, u8;
	AtomicU16, u16;
	AtomicU32, u32;
	AtomicI8, i8;
	AtomicI16, i16;
	AtomicI32, i32;
	AtomicUsize, usize;
	AtomicIsize, isize
);

#[cfg(target_pointer_width = "64")]
atomic_load_impl!(AtomicU64, u64; AtomicI64, i64);
