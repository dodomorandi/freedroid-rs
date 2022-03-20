use std::{
    ffi::CStr,
    fmt,
    ops::{Deref, Index},
};

#[derive(Debug, Clone, Copy, Eq)]
pub struct ArrayCString<const N: usize = 1>([u8; N]);

impl<const N: usize> ArrayCString<N> {
    #[inline]
    pub const fn new() -> Self {
        assert!(N > 0);
        Self([0; N])
    }

    pub fn len(&self) -> usize {
        // SAFETY:
        // It is not possible to construct an ArrayCString without a string terminator.
        unsafe { self.0.iter().position(|&c| c == 0).unwrap_unchecked() }
    }

    pub fn clear(&mut self) {
        self.0[0] = b'\0';
    }

    pub fn try_push_cstr(&mut self, s: impl AsRef<CStr>) -> Result<(), Error> {
        let cur_len = self.len();
        let s = s.as_ref().to_bytes_with_nul();
        let new_len_with_nul = cur_len
            .checked_add(s.len())
            .filter(|&new_len| new_len < N)
            .ok_or(Error)?;

        self.0[cur_len..new_len_with_nul].copy_from_slice(s);
        Ok(())
    }

    pub fn try_set(&mut self, s: impl AsRef<CStr>) -> Result<(), Error> {
        let s = s.as_ref().to_bytes_with_nul();
        let new_len_with_nul = s.len();
        if new_len_with_nul >= N {
            return Err(Error);
        }

        self.0[..new_len_with_nul].copy_from_slice(s);
        Ok(())
    }

    pub fn try_set_slice(&mut self, s: impl AsRef<[u8]>) -> Result<(), Error> {
        let s = s.as_ref();
        let new_len = s.len();
        if new_len + 1 >= N {
            return Err(Error);
        }

        self.0[..new_len].copy_from_slice(s);
        self.0[new_len] = b'\0';
        Ok(())
    }

    pub fn try_push_str(&mut self, s: impl AsRef<str>) -> Result<(), Error> {
        let cur_len = self.len();
        let s = s.as_ref();
        let new_len = cur_len
            .checked_add(s.len())
            .filter(|&new_len| new_len < N - 1)
            .ok_or(Error)?;

        self.0[cur_len..new_len].copy_from_slice(s.as_bytes());
        self.0[new_len] = b'\0';
        Ok(())
    }

    #[inline]
    pub fn push_cstr(&mut self, s: impl AsRef<CStr>) {
        self.try_push_cstr(s).expect("reached end of array buffer")
    }

    #[inline]
    pub fn set(&mut self, s: impl AsRef<CStr>) {
        self.try_set(s).expect("reached end of array buffer")
    }

    #[inline]
    pub fn set_slice(&mut self, s: impl AsRef<[u8]>) {
        self.try_set_slice(s).expect("reached end of array buffer")
    }

    #[inline]
    pub fn push_str(&mut self, s: impl AsRef<str>) {
        self.try_push_str(s).expect("reached end of array buffer")
    }

    pub fn truncate(&mut self, new_len: usize) {
        if let Some(c) = self.0.get_mut(new_len) {
            *c = b'\0';
        }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }
}

impl<const N: usize> Index<usize> for ArrayCString<N> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        let len = self.len();
        if index >= len {
            panic!("index {index} out of bound on an ArrayCString<{N}> of len {len}");
        }
        &self.0[index]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Error;

impl<const N: usize> Deref for ArrayCString<N> {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // It is not possible to construct an ArrayCString without a string terminator.
        unsafe {
            let slice_with_nul = self
                .0
                .split_inclusive(|&c| c == 0)
                .next()
                .unwrap_unchecked();
            CStr::from_bytes_with_nul_unchecked(slice_with_nul)
        }
    }
}

impl<const N: usize> fmt::Write for ArrayCString<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_push_str(s).map_err(|_| fmt::Error)
    }
}

impl<const N: usize> Default for ArrayCString<N> {
    fn default() -> Self {
        assert!(N > 0);
        Self([0; N])
    }
}

impl<const N: usize> PartialEq for ArrayCString<N> {
    fn eq(&self, other: &Self) -> bool {
        &**self == &**other
    }
}
