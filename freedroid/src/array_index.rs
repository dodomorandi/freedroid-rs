use std::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenericArrayIndex<T, const N: usize>(T);

pub type ArrayIndex<const N: usize> = GenericArrayIndex<usize, N>;

impl<T, U, const N: usize> Index<GenericArrayIndex<T, N>> for [U; N]
where
    [U; N]: Index<T>,
    T: SliceIndex<[U], Output = U>,
{
    type Output = U;

    fn index(&self, index: GenericArrayIndex<T, N>) -> &Self::Output {
        // SAFETY: value inside GenricArrayIndex must be < N
        unsafe { self.get_unchecked(index.0) }
    }
}

impl<T, U, const N: usize> IndexMut<GenericArrayIndex<T, N>> for [U; N]
where
    [U; N]: IndexMut<T> + Index<GenericArrayIndex<T, N>, Output = U>,
    T: SliceIndex<[U], Output = U>,
{
    fn index_mut(&mut self, index: GenericArrayIndex<T, N>) -> &mut Self::Output {
        // SAFETY: value inside GenricArrayIndex must be < N
        unsafe { self.get_unchecked_mut(index.0) }
    }
}

impl<const N: usize> ArrayIndex<N> {
    pub const fn new(index: usize) -> Self {
        assert!(index < N, "creating an out of bound array index");

        Self(index)
    }
}
