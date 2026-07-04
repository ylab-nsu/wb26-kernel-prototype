pub mod address_space;
pub mod page_table;

use bitfield_struct::bitfield;

use crate::arch::macros::impl_address;

pub const PAGE_SIZE: usize = 4096;

#[bitfield(u64)]
pub struct Sv39VirtualAddress {
    #[bits(12)]
    offset: usize,

    #[bits(9)]
    vpn_0: usize,
    #[bits(9)]
    vpn_1: usize,
    #[bits(9)]
    vpn_2: usize,

    #[bits(25)]
    __: usize,
}

impl_address!(Sv39VirtualAddress, u64);

#[bitfield(u64)]
pub struct Sv39PhysicalAddress {
    #[bits(12)]
    offset: usize,

    #[bits(9)]
    ppn_0: usize,
    #[bits(9)]
    ppn_1: usize,
    #[bits(26)]
    ppn_2: usize,

    #[bits(8)]
    __: usize,
}

impl_address!(Sv39PhysicalAddress, u64);
