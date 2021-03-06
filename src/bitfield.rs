//! Definitions of bitfield things for hashmap neighbourhoods.
use std::iter::Iterator;
use std::ops::{BitAnd, BitOr, Shr};

/// A bit field trait for use in hashmap buckets. See the `bitfield` method of `BiMapBuilder` for
/// more information.
pub trait BitField: BitAnd<Output = Self> + BitOr<Output = Self> + Copy + Sized {
    /// See the documentation for the `iter` function.
    type Iter: Iterator<Item = usize>;

    /// Should return a constant value describing how big the bitfield of this type is.
    fn size() -> usize;

    /// Should return a bitfield that is all zeroes, except for a single one at a given index.
    fn one_at(index: usize) -> Self;

    /// Should return a bitfield that is all ones, except for a single zero at a given index.
    fn zero_at(index: usize) -> Self;

    /// Return an iterator that iterates through the bitfield, returning the indexes within the
    /// bitfield that have 1s in them, in order from least significant to most significant.
    fn iter(&self) -> Self::Iter;

    /// Is the bitfield currently full?
    fn full(&self) -> bool;
}

mod private {
    use super::{BitField, BitFieldIterator};

    use std::ops::{BitAnd, BitOr, Not, Shl, Shr};

    /// Helper trait to reduce code duplication when implementing Bitfield for integer types.
    pub trait BitSized {
        /// Returns how many bits are in the type.
        fn size() -> usize;
    }

    impl BitSized for u8 {
        fn size() -> usize {
            8
        }
    }

    impl BitSized for u16 {
        fn size() -> usize {
            16
        }
    }

    impl BitSized for u32 {
        fn size() -> usize {
            32
        }
    }

    impl BitSized for u64 {
        fn size() -> usize {
            64
        }
    }

    impl<T> BitField for T
    where
        T: BitSized
            + BitAnd<Output = T>
            + BitOr<Output = T>
            + Eq
            + Not<Output = T>
            + Shl<usize, Output = T>
            + Shr<usize, Output = T>
            + From<u8>
            + Copy,
    {
        type Iter = BitFieldIterator<T>;

        fn size() -> usize {
            <T as BitSized>::size()
        }

        fn one_at(index: usize) -> Self {
            Self::from(1) << index
        }

        fn zero_at(index: usize) -> Self {
            !Self::one_at(index)
        }

        fn iter(&self) -> Self::Iter {
            BitFieldIterator(*self, 0)
        }

        fn full(&self) -> bool {
            *self == Self::one_at(0) | Self::zero_at(0)
        }
    }
}

/// An iterator over the active bits in a bitfield.
pub struct BitFieldIterator<T>(T, usize);

impl<T> Iterator for BitFieldIterator<T>
where
    T: Eq + BitAnd<Output = T> + Shr<usize, Output = T> + From<u8> + Copy,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut BitFieldIterator(ref mut bitfield, ref mut index) = self;

        if *bitfield == T::from(0) {
            None
        } else {
            while T::from(1) & *bitfield == T::from(0) {
                *bitfield = *bitfield >> 1;
                *index += 1;
            }
            *bitfield = *bitfield >> 1;
            *index += 1;
            Some(*index - 1)
        }
    }
}

/// The default bitfield type.
pub type DefaultBitField = u32;

#[cfg(test)]
mod test {
    use super::BitField;

    use quickcheck::TestResult;

    quickcheck! {
        fn one_at(index: usize) -> TestResult {
            if index >= u64::size() {
                TestResult::discard()
            } else {
                TestResult::from_bool(u64::one_at(index).iter().collect::<Vec<_>>() == vec![index])
            }
        }
    }

    quickcheck! {
        fn iterator_results_are_ordered(input: u32) -> bool {
            let bits: Vec<_> = input.iter().collect();
            bits[..].windows(2).all(|window| {
                window[0] < window[1]
            })
        }
    }

    quickcheck! {
        fn iterator_results_equal_number(input: u32) -> bool {
            input == input.iter()
                .map(|x| 1 << x)
                .fold(0, |x, y| x + y)
        }
    }
}
