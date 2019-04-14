# atomic_bitfield

## Atomic Bitfield
 Provides a bitfield abstraction for the core atomic types. This crate is `no_std` compatible
by default, and does not itself use any `unsafe` code.

__Note__: On `stable` this crate assumes the presence of the following atomics
which may cause compilation to fail on certain platforms.
* `Atomic{U,I}32` and smaller
* `Atomic{U,I}size`
* `Atomic{U,I}64` on 64 bit platforms

The `nightly` feature of this crate enables `target_has_atomic` and uses
that instead to detect which atomic types are available.

## Usage Example
```rust
use core::sync::atomic::{AtomicU8, Ordering::Relaxed};
use atomic_bitfield::AtomicBitField as _;

let flags = AtomicU8::new(0b1000);

let prev_state = flags.set_bit(0, Relaxed);
assert_eq!(prev_state, false);
assert_eq!(flags.load(Relaxed), 0b1001);

let prev_state = flags.toggle_bit(3, Relaxed);
assert_eq!(prev_state, true);
assert_eq!(flags.load(Relaxed), 0b0001);

let prev_state = flags.swap_bit(0, false, Relaxed);
assert_eq!(prev_state, true);
assert_eq!(flags.load(Relaxed), 0b0000);
```

License: MIT
