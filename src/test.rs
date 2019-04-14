#[macro_use(quickcheck)]

use super::AtomicBitField;
use std::{ops::BitAnd, sync::atomic::*};

#[test]
fn test_bit_len() {
	assert_eq!(AtomicU8::bit_len(), 8);
	assert_eq!(AtomicU16::bit_len(), 16);
	assert_eq!(AtomicU32::bit_len(), 32);

	assert_eq!(AtomicI8::bit_len(), 8);
	assert_eq!(AtomicI16::bit_len(), 16);
	assert_eq!(AtomicI32::bit_len(), 32);
}

fn clamp_input<'a, T, I>(iter: I, max: T) -> impl Iterator<Item = T> + 'a + Clone
where
	I: Iterator<Item = &'a T> + 'a + Clone,
	T: BitAnd<Output = T> + Sized + Copy + 'a,
{
	iter.map(move |val| val.bitand(max))
}

fn atomic_map<'a, A, I, T>(iter: I) -> impl Iterator<Item = A> + 'a + Clone
where
	I: Iterator<Item = &'a T> + 'a + Clone,
	T: 'a + Copy,
	A: From<T>
{
	iter.map(|&val| A::from(val))
}

quickcheck::quickcheck! {
	fn bit_flipping_u8(ints: Vec<u8>, bits: Vec<u8>) -> bool {

		if ints.len() == 0 || bits.len() == 0 {
			return true;
		}

		// let aints: Vec<_> = ints.iter().map(|&int| AtomicU8::new(int)).collect();
		let aints = atomic_map::<AtomicU8, _, _>(ints.iter());

		let mapped_bits = clamp_input(bits.iter(), 7);

		let prev_state: Vec<_> = aints.clone().zip(mapped_bits.clone()).map(|(aint, bit)| aint.set_bit(bit as _, Ordering::Relaxed)).collect();

		let flipped_ints: Vec<_> = aints.map(AtomicU8::into_inner).collect();

		let flipped_aints: Vec<_> = flipped_ints
			.iter()
			.map(|&fint| AtomicU8::new(fint))
			.collect();

		flipped_aints.iter().zip(mapped_bits).zip(prev_state).for_each(|((faint, bit), prev)| {
			faint.swap_bit(bit as _, prev, Ordering::Relaxed);
		});

		let flipflipped_aints: Vec<_> = flipped_aints.into_iter().map(AtomicU8::into_inner).collect();

		flipflipped_aints == ints
	}
}
