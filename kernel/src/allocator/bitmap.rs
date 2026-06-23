use bitmap_allocator::BitAlloc;

use crate::{allocator::AllocatorError, arch::traits::TargetAddress};

pub struct BitmapMemoryAllocator<A: TargetAddress, B: BitAlloc, const G: usize> {
    bitmap: B,
    base_addr: A,
    size: usize,
}

impl<A: TargetAddress, B: BitAlloc, const G: usize> BitmapMemoryAllocator<A, B, G> {
    pub unsafe fn new(base: A, size: usize) -> Self {
        debug_assert!(
            G.is_power_of_two(),
            "Granularity should always be a power of two"
        );
        debug_assert!(
            size.is_multiple_of(G),
            "Size should be a multiple of granularity"
        );

        let mut allocator = BitmapMemoryAllocator {
            bitmap: B::DEFAULT,
            base_addr: base,
            size,
        };

        let size_units = size / G;

        allocator.bitmap.insert(0..size_units);

        allocator
    }

    pub fn alloc_contiguous(&mut self, size: usize) -> Result<(A, usize), AllocatorError> {
        let size_rounded = size.next_multiple_of(G);
        let size_units = size_rounded / G;

        let index = self
            .bitmap
            .alloc_contiguous(None, size_units, 0)
            .ok_or(AllocatorError::NotEnoughMemory)?;

        Ok((self.base_addr.byte_add(index * G), size_rounded))
    }

    pub fn alloc_contiguous_aligned(
        &mut self,
        size: usize,
        alignment: usize,
    ) -> Result<(A, usize), AllocatorError> {
        let max_alignment_log2 = self
            .base_addr
            .try_into()
            .unwrap()
            .trailing_zeros()
            .clamp(0, usize::BITS - 1);

        let max_alignment = 1 << max_alignment_log2;

        if !alignment.is_power_of_two() || alignment > max_alignment {
            return Err(AllocatorError::InvalidAlignment);
        }

        let alignment = if alignment < G { G } else { alignment };

        let size_rounded = size.next_multiple_of(alignment);
        debug_assert!(size_rounded.is_multiple_of(G));

        let size_units = size_rounded / G;

        let align_log2 = alignment.trailing_zeros() - G.trailing_zeros();

        let index = self
            .bitmap
            .alloc_contiguous(None, size_units, align_log2 as usize)
            .ok_or(AllocatorError::NotEnoughMemory)?;

        Ok((self.base_addr.byte_add(index * G), size_rounded))
    }

    pub fn alloc_contiguous_at(
        &mut self,
        addr: A,
        size: usize,
    ) -> Result<(A, usize), AllocatorError> {
        if addr < self.base_addr {
            return Err(AllocatorError::InvalidAddress);
        }

        let offset = addr.byte_offset_from_unsigned(self.base_addr);

        if !offset.is_multiple_of(G) {
            return Err(AllocatorError::InvalidAlignment);
        }

        let size_rounded = size.next_multiple_of(G);
        let size_units = size_rounded / G;

        if offset + size_rounded > self.size {
            return Err(AllocatorError::InvalidAddress);
        }

        let index = self
            .bitmap
            .alloc_contiguous(Some(offset / G), size_units, 0)
            .ok_or(AllocatorError::AlreadyOccupied)?;

        Ok((self.base_addr.byte_add(index * G), size_rounded))
    }

    pub unsafe fn dealloc_contiguous(&mut self, addr: A, size: usize) {
        debug_assert!(addr >= self.base_addr);

        let offset = addr.byte_offset_from_unsigned(self.base_addr);

        debug_assert!(offset.is_multiple_of(G));
        debug_assert!(size.is_multiple_of(G));
        debug_assert!(offset + size <= self.size);

        let size_units = size / G;

        let result = self.bitmap.dealloc_contiguous(offset / G, size_units);

        debug_assert!(result, "Double free");
    }
}
