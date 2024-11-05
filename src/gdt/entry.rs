use crate::gdt::access::Access;
use crate::gdt::flag::Flag;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    base_31_24: u8,
    flags_limit_16_19: u8,
    access: u8,
    base_16_23: u8,
    base_0_15: u16,
    limit_0_15: u16,
}

impl Entry {
    pub fn new(base: u32, limit: u32, flag: Flag, access: Access) -> Self {
        Entry {
            base_31_24: ((base & 0xFF000000) >> 24) as u8,
            flags_limit_16_19: flag.to_u8() | ((limit & 0x000F0000) >> 16) as u8,
            access: access.to_u8(),
            base_16_23: ((base & 0x000F0000) >> 16) as u8,
            base_0_15: (base & 0x0000FFFF) as u16,
            limit_0_15: (limit & 0x0000FFFF) as u16,
        }
    }
    pub fn new_zero() -> Self {
        Entry {
            base_31_24: 0,
            flags_limit_16_19: 0,
            access: 0,
            base_16_23: 0,
            base_0_15: 0,
            limit_0_15: 0,
        }
    }

    pub fn to_u64(&self) -> u64 {
        let mut result: u64 = 0;

        result |= (self.base_31_24 as u64) << 56;
        result |= (self.flags_limit_16_19 as u64) << 48;
        result |= (self.access as u64) << 40;
        result |= (self.base_16_23 as u64) << 32;
        result |= (self.base_0_15 as u64) << 16;
        result |= self.limit_0_15 as u64;

        result
    }
}