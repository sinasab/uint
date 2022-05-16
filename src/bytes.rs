// TODO: Use u64::from_{be/le}_bytes().
// TODO: Make `const fn`s when `const_for` is stable.

use crate::Uint;

// TODO: Use `Self::BYTES` instead of a generic argument and runtime assertion.
// Blocked by Rust issue [#60551](https://github.com/rust-lang/rust/issues/60551).
impl<const BITS: usize, const LIMBS: usize> Uint<BITS, LIMBS> {
    /// The size of this integer type in bytes. Note that some bits may be
    /// forced zero if BITS is not cleanly divisible by eight.
    pub const BYTES: usize = (BITS + 7) / 8;

    /// Creates a new integer from a little endian stream of bytes.
    #[must_use]
    #[allow(clippy::cast_lossless)]
    fn try_from_le_byte_iter<I>(iter: I) -> Option<Self>
    where
        I: Iterator<Item = u8>,
    {
        let mut limbs = [0; LIMBS];
        for (i, byte) in iter.enumerate() {
            if byte == 0 {
                continue;
            }
            let limb_index = i / 8;
            if limb_index >= Self::LIMBS {
                return None;
            }
            let byte_index = i % 8;
            limbs[limb_index] += (byte as u64) << (byte_index * 8);
        }
        if Self::LIMBS > 0 && limbs[Self::LIMBS - 1] > Self::MASK {
            return None;
        }
        Some(Self::from_limbs(limbs))
    }

    /// Creates a new integer from a big endian slice of bytes.
    ///
    /// The slice is interpreted as a big endian number. Leading zeros
    /// are ignored. The slice can be any length.
    ///
    /// Returns [`None`] if the value is larger than fits the [`Uint`].
    #[must_use]
    pub fn try_from_be_slice(bytes: &[u8]) -> Option<Self> {
        Self::try_from_le_byte_iter(bytes.iter().copied().rev())
    }

    /// Creates a new integer from a little endian slice of bytes.
    ///
    /// The slice is interpreted as a little endian number. Leading zeros
    /// are ignored. The slice can be any length.
    ///
    /// Returns [`None`] if the value is larger than fits the [`Uint`].
    #[must_use]
    pub fn try_from_le_slice(bytes: &[u8]) -> Option<Self> {
        Self::try_from_le_byte_iter(bytes.iter().copied())
    }

    /// # Panics
    ///
    /// Panics if the array is not exactly [`Self::BYTES`] long. Ideally this
    /// would be a compile time error, but this is blocked by Rust issue
    /// [#60551].
    ///
    /// [#60551]: https://github.com/rust-lang/rust/issues/60551
    ///
    /// Panics if the value is too large for the bit-size of the Uint.
    #[must_use]
    #[track_caller]
    pub fn from_be_bytes<const BYTES: usize>(bytes: [u8; BYTES]) -> Self {
        assert_eq!(BYTES, Self::BYTES);
        match Self::try_from_be_slice(&bytes) {
            Some(uint) => uint,
            None => panic!("Value too large for Uint<{}>", BITS),
        }
    }

    /// # Panics
    ///
    /// Panics if the array is not exactly [`Self::BYTES`] long. Ideally this
    /// would be a compile time error, but this is blocked by Rust issue
    /// [#60551].
    ///
    /// [#60551]: https://github.com/rust-lang/rust/issues/60551
    ///
    /// Panics if the value is too large for the bit-size of the Uint.
    #[must_use]
    #[track_caller]
    pub fn from_le_bytes<const BYTES: usize>(bytes: [u8; BYTES]) -> Self {
        assert_eq!(BYTES, Self::BYTES);
        match Self::try_from_le_slice(&bytes) {
            Some(uint) => uint,
            None => panic!("Value too large for Uint<{}>", BITS),
        }
    }

    /// # Panics
    ///
    /// Panics if the array is not exactly [`Self::BYTES`] long. Ideally this
    /// would be a compile time error, but this is blocked by Rust issue
    /// [#60551].
    ///
    /// [#60551]: https://github.com/rust-lang/rust/issues/60551
    #[must_use]
    pub fn to_be_bytes<const BYTES: usize>(&self) -> [u8; BYTES] {
        assert_eq!(BYTES, Self::BYTES);
        let mut bytes = [0; BYTES];
        for (chunk, limb) in bytes.rchunks_mut(8).zip(self.as_limbs().iter().copied()) {
            chunk.copy_from_slice(&limb.to_be_bytes()[(8 - chunk.len())..]);
        }
        bytes
    }

    /// # Panics
    ///
    /// Panics if the array is not exactly [`Self::BYTES`] long. Ideally this
    /// would be a compile time error, but this is blocked by Rust issue
    /// [#60551].
    ///
    /// [#60551]: https://github.com/rust-lang/rust/issues/60551
    #[must_use]
    pub fn to_le_bytes<const BYTES: usize>(&self) -> [u8; BYTES] {
        assert_eq!(BYTES, Self::BYTES);
        let mut bytes = [0; BYTES];
        for (chunk, limb) in bytes.chunks_mut(8).zip(self.as_limbs().iter().copied()) {
            chunk.copy_from_slice(&limb.to_le_bytes()[..chunk.len()]);
        }
        bytes
    }
}

/// Number of bytes required to represent the given number of bits.
///
/// This needs to be public because it is used in the `Uint` type,
/// specifically in the [`to_be_bytes()`][Uint::to_be_bytes] and related
/// functions.
#[must_use]
pub const fn nbytes(bits: usize) -> usize {
    (bits + 7) / 8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{const_for, nlimbs};
    use proptest::proptest;

    const N: Uint<128, 2> =
        Uint::from_limbs([0x7890_1234_5678_9012_u64, 0x1234_5678_9012_3456_u64]);
    const BE: [u8; 16] = [
        0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90,
        0x12,
    ];
    const LE: [u8; 16] = [
        0x12, 0x90, 0x78, 0x56, 0x34, 0x12, 0x90, 0x78, 0x56, 0x34, 0x12, 0x90, 0x78, 0x56, 0x34,
        0x12,
    ];

    const K: Uint<72, 2> = Uint::from_limbs([0x3456_7890_1234_5678_u64, 0x12_u64]);
    const KBE: [u8; 9] = [0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78];
    const KLE: [u8; 9] = [0x78, 0x56, 0x34, 0x12, 0x90, 0x78, 0x56, 0x34, 0x12];

    #[test]
    fn test_from_bytes() {
        assert_eq!(Uint::<0, 0>::from_be_bytes([]), Uint::ZERO);
        assert_eq!(Uint::<0, 0>::from_le_bytes([]), Uint::ZERO);
        assert_eq!(
            Uint::<12, 1>::from_be_bytes([0x01, 0x23]),
            Uint::from(0x0123)
        );
        assert_eq!(
            Uint::<12, 1>::from_le_bytes([0x23, 0x01]),
            Uint::from(0x0123)
        );
        assert_eq!(
            Uint::<16, 1>::from_be_bytes([0x12, 0x34]),
            Uint::from(0x1234)
        );
        assert_eq!(
            Uint::<16, 1>::from_le_bytes([0x34, 0x12]),
            Uint::from(0x1234)
        );
        assert_eq!(Uint::from_be_bytes(BE), N);
        assert_eq!(Uint::from_le_bytes(LE), N);
        assert_eq!(Uint::from_be_bytes(KBE), K);
        assert_eq!(Uint::from_le_bytes(KLE), K);
    }

    #[test]
    fn test_to_bytes() {
        assert_eq!(Uint::<0, 0>::ZERO.to_le_bytes(), [0_u8; 0]);
        assert_eq!(Uint::<0, 0>::ZERO.to_be_bytes(), [0_u8; 0]);
        assert_eq!(Uint::<12, 1>::from(0x0123_u64).to_le_bytes(), [0x23, 0x01]);
        assert_eq!(Uint::<12, 1>::from(0x0123_u64).to_be_bytes(), [0x01, 0x23]);
        assert_eq!(Uint::<16, 1>::from(0x1234_u64).to_le_bytes(), [0x34, 0x12]);
        assert_eq!(Uint::<16, 1>::from(0x1234_u64).to_be_bytes(), [0x12, 0x34]);
        assert_eq!(K.to_be_bytes(), KBE);
        assert_eq!(K.to_le_bytes(), KLE);
    }

    #[test]
    fn test_bytes_roundtrip() {
        const_for!(BITS in SIZES {
            const LIMBS: usize = nlimbs(BITS);
            const BYTES: usize = nbytes(BITS);
            proptest!(|(value: Uint<BITS, LIMBS>)| {
                assert_eq!(value, Uint::from_be_bytes(value.to_be_bytes::<BYTES>()));
                assert_eq!(value, Uint::from_le_bytes(value.to_le_bytes::<BYTES>()));
            });
        });
    }
}
