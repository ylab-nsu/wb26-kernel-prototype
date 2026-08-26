//! Executable file image resident in RAM (the `bprm->file` analogue).
//!
//! Phase-1 backing: the whole file sits at a fixed physical address in RAM
//! (loaded by QEMU `-device loader`). The kernel reads it directly through
//! the identity mapping of the first 4 GiB (`init_mmu`). This keeps the same
//! `ReadRef` flow as the harness — the loader code does not change if the
//! backing later becomes chunked disk reads.

use core::slice;

use object::read::ReadRef;

use crate::exec::ExecError;

/// A file image backed by a fixed RAM range.
///
/// No size bound is tracked: the image is assumed to be a well-formed file at
/// `base`, and reads are served straight from memory.
#[derive(Clone, Copy)]
pub struct Image {
    base: *const u8,
}

impl Image {
    pub fn new(base: *const u8) -> Image {
        Image { base }
    }

    /// Read `size` bytes from file offset `offset` directly into `dst`.
    ///
    /// RAM backing: a plain copy from the resident image. A future disk
    /// backing would read straight into `dst` instead.
    pub fn read_into(&self, offset: u64, dst: *mut u8, size: usize) -> Result<(), ExecError> {
        let src = unsafe { self.base.add(offset as usize) };
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, size);
        }
        Ok(())
    }
}

impl<'a> ReadRef<'a> for Image {
    fn len(self) -> Result<u64, ()> {
        // Unbounded: the object parser never asks past the file's own fields.
        Ok(u64::MAX)
    }

    fn read_bytes_at(self, offset: u64, size: u64) -> Result<&'a [u8], ()> {
        let p = unsafe { self.base.add(offset as usize) };
        Ok(unsafe { slice::from_raw_parts(p, size as usize) })
    }

    fn read_bytes_at_until(
        self,
        range: core::ops::Range<u64>,
        delimiter: u8,
    ) -> Result<&'a [u8], ()> {
        let size = range.end.checked_sub(range.start).ok_or(())?;
        let bytes = self.read_bytes_at(range.start, size)?;
        match bytes.iter().position(|&b| b == delimiter) {
            Some(i) => Ok(&bytes[..i]),
            None => Err(()),
        }
    }
}