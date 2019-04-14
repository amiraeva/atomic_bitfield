#[macro_use(quickcheck)]

use super::AtomicBitField;
use std::sync::atomic::*;

#[test]
fn test_bit_len() {
	assert_eq!(AtomicU8::bit_len(), 8);
	assert_eq!(AtomicU16::bit_len(), 16);
	assert_eq!(AtomicU32::bit_len(), 32);

	assert_eq!(AtomicI8::bit_len(), 8);
	assert_eq!(AtomicI16::bit_len(), 16);
	assert_eq!(AtomicI32::bit_len(), 32);
}

quickcheck::quickcheck! {
	fn bit_flipping_u8(ints: Vec<u8>, bits: Vec<u8>) -> bool {
		
		if ints.len() == 0 || bits.len() == 0 {
			return true;
		}

		let aints: Vec<_> = ints.iter().map(|&int| AtomicU8::new(int)).collect();
		let mapped_bits = bits.iter().map(|bit| bit & 7);

		let prev_state: Vec<_> = aints.iter().zip(mapped_bits.clone()).map(|(aint, bit)| aint.set_bit(bit as _, Ordering::Relaxed)).collect();

		let flipped_ints: Vec<_> = aints.into_iter().map(AtomicU8::into_inner).collect();

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
