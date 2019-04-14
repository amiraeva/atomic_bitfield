#[macro_use(quickcheck)]

use super::AtomicBitField;

use std::{
	mem::size_of,
	ops::{BitAnd, Sub},
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

fn clamp_input<'a, T, I>(
	iter: I,
	max: T,
) -> impl Iterator<Item = <T as BitAnd<<T as Sub<u8>>::Output>>::Output> + 'a + Clone
where
	I: Iterator<Item = &'a T> + 'a + Clone,
	T: BitAnd<<T as Sub<u8>>::Output> + Sized + Copy + 'a + Sub<u8>,
{
	iter.map(move |val| val.bitand(max - 1))
}

fn atomic_map<'a, A, I, T>(iter: I) -> impl Iterator<Item = A> + 'a + Clone
where
	I: Iterator<Item = &'a T> + 'a + Clone,
	T: 'a + Copy,
	A: From<T> + AtomicBitField,
{
	iter.map(|&val| A::from(val))
}

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

// sets a bit, and swaps it back, outputs if the result is the same as the original
fn bit_flipping<A, Int>(test_vals: Vec<(Int, u8)>) -> bool
where
	A: AtomicBitField + AtomicLoad + From<Int>,
	Int: Copy + PartialEq + std::fmt::Binary,
	A::Inner: Into<Int> + std::fmt::Binary,
{

	let ints = test_vals.iter().map(|(int, _)| int);
	let tmp = test_vals.iter().map(|(_, bit)| bit);

	let bits = clamp_input(tmp, AtomicU8::bit_len() as _);
	let atomic_ints = atomic_map::<A, _, _>(ints.clone());

	let flippedflipped_ints = atomic_ints
		.inspect(|aint| print!("pre set: {:#b}", aint.load()))
		.zip(bits)
		.map(|(aint, bit)| {
			let prev = aint.set_bit(bit as _, Ordering::Relaxed);
			(aint, bit, prev)
		})
		.inspect(|(val, bit, _)| print!(", post set: {:#b}, bit: {}", val.load(), bit))
		.map(|(aint, bit, prev)| {
			aint.swap_bit(bit as _, prev, Relaxed);
			aint
		})
		.map(|a| a.load().into())
		.inspect(|val| println!(", post swap: {:#b}", val));

	ints.map(|&x| Into::<Int>::into(x)).eq(flippedflipped_ints)
}

// toggles a bit, toggles it back, outputs if the result is the same as the original
fn bit_toggling<A, Int>(test_vals: Vec<(Int, u8)>) -> bool 
where
	A: AtomicBitField + AtomicLoad<Inner = Int> + From<Int>,
	Int: Copy + PartialEq
{
	let ints = test_vals.iter().map(|(int, _)| *int);
	
	let toggle = |(val, bit): &(A, u8)| {
		val.toggle_bit(*bit as _, Relaxed);
	};

	let to_int = |(val, _): (A, _)| val.load();

	test_vals.iter()
		.map(|(val, bit)| (A::from(*val), bit & (A::bit_len() - 1) as u8))
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
				quickcheck::quickcheck! {
					fn $primitive_t(test_vals: Vec<($primitive_t, u8)>) -> bool {
						bit_flipping::<$atomic_t, _>(test_vals)
					}
				}
			)*
		}

		mod $toggle {
			use super::*;
			$(
				quickcheck::quickcheck! {
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
