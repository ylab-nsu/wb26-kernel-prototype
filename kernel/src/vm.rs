use core::fmt::{Display, Write};

use bitfield_struct::bitfield;

#[bitfield(u8)]
pub struct MappingPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,

    #[bits(5)]
    __: usize,
}

impl MappingPermissions {
    pub const fn ro() -> Self {
        Self::new()
            .with_read(true)
    }

    pub const fn rw() -> Self {
        Self::new()
            .with_read(true)
            .with_write(true)
    }

    pub const fn rx() -> Self {
        Self::new()
            .with_read(true)
            .with_execute(true)
    }

    pub const fn rwx() -> Self {
        Self::new()
            .with_read(true)
            .with_write(true)
            .with_execute(true)
    }
}

impl Display for MappingPermissions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char('[').unwrap();

        if self.read() {
            f.write_char('R').unwrap();
        }
        if self.write() {
            f.write_char('W').unwrap();
        }
        if self.execute() {
            f.write_char('X').unwrap();
        }

        f.write_char(']').unwrap();

        Ok(())
    }
}

#[bitfield(u8)]
pub struct MappingFlags {
    pub user: bool,
    pub global: bool,
    pub accessed: bool,
    pub dirty: bool,

    #[bits(4)]
    __: usize,
}

impl Display for MappingFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char('[').unwrap();
        
        if self.user() {
            f.write_char('U').unwrap();
        }
        if self.global() {
            f.write_char('G').unwrap();
        }
        if self.accessed() {
            f.write_char('A').unwrap();
        }
        if self.dirty() {
            f.write_char('D').unwrap();
        }

        f.write_char(']').unwrap();

        Ok(())
    }
}
