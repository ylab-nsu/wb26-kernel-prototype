use ::alloc::vec::Vec;
use alloc::boxed::Box;
use core::ops::{Deref, DerefMut};
use crate::paging::alloc::PhysicalPage;

type Page = [u8; 4096];

// struct MappedPagePool {
//     pages: Vec<PhysicalPage>,
//     bitmap: (),
//     // ptr: NonNull<Page>,
// }
//
// impl MappedPagePool {
//     fn get_page(&mut self) -> MappedPage {
//         MappedPage {
//             ptr: 0x8000_0000usize as *mut [u8; 4096],
//         }
//     }
// }

// struct MappedPage {
//     // ptr: NonNull<MaybeUninit<[u8; 4096]>>
//     ptr: *mut [u8; 4096],
// }
//
// impl Deref for MappedPage {
//     type Target = [u8; 4096];
//
//     fn deref(&self) -> &Self::Target {
//         unsafe { &*self.ptr }
//     }
// }
//
// impl DerefMut for MappedPage {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         unsafe { &mut *self.ptr }
//     }
// }

struct MappedPage<'a> {
    // ptr: NonNull<MaybeUninit<[u8; 4096]>>
    ptr: &'a mut [u8; 4096],
}

impl Deref for MappedPage<'_> {
    type Target = [u8; 4096];

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr }
    }
}

impl DerefMut for MappedPage<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr }
    }
}

fn a() {
    // let mut mpp = MappedPagePool { pages: vec![], bitmap: (), ptr: () };
    // let mut mp = mpp.get_page();
    //
    // let x = u64::from_ne_bytes(mp[0..7].try_into().unwrap());
    //
    // let xx = 123u64;
    // let hh = xx.to_ne_bytes();
    // mp[0..].copy_from_slice(&hh);
}
