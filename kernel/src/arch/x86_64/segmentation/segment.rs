use crate::arch::x86_64::segmentation::tss::TaskStateSegment;
use crate::arch::x86_64::PrivilegeLevel;
use core::mem::size_of;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AccessByte {
    value: u8,
}

impl AccessByte {
    const NULL: Self = AccessByte::new(false, PrivilegeLevel::Ring0);

    const fn new(present: bool, privilege: PrivilegeLevel) -> Self {
        let mut value = 0;
        value |= (present as u8) << 7;
        value |= (privilege as u8) << 5;

        Self { value }
    }

    fn privilege(&self) -> PrivilegeLevel {
        let value = self.value >> 5 & 3;
        value.into()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UserAccessByte {
    value: AccessByte,
}

impl UserAccessByte {
    const NULL: Self = UserAccessByte {
        value: AccessByte::NULL,
    };

    const fn new(access: AccessByte, executable: bool, rw: bool, dc: bool) -> Self {
        let mut value_byte: u8 = access.value;
        value_byte |= 1 << 4; // to make it as a user segment.
        value_byte |= (executable as u8) << 3;
        value_byte |= (dc as u8) << 2;
        value_byte |= (rw as u8) << 1;

        Self {
            value: AccessByte { value: value_byte },
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum SystemAccessByteKind {
    Ldt = 0x2,
    TssAvailable16Bit = 0x1,
    TssBusy16Bit = 0x3,
    TssAvailable = 0x9,
    TssBusy = 0xB,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SystemAccessByte {
    value: AccessByte,
}

impl SystemAccessByte {
    const fn new(access: AccessByte, kind: SystemAccessByteKind) -> Self {
        let mut value_byte: u8 = access.value;
        value_byte |= kind as u8;

        Self {
            value: AccessByte { value: value_byte },
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NormalSegment<A> {
    limit: u16,
    base: u16,
    middle_base: u8,
    access_byte: A,
    attrs_higher: u16,
}

impl<A> NormalSegment<A> {
    const fn new(
        base: u32,
        limit: u32,
        long_mode: bool,
        bit_32_mode: bool,
        granularity: bool,
        access: A,
    ) -> Self {
        let mut descriptor = Self {
            limit: 0,
            base: 0,
            middle_base: 0,
            access_byte: access,
            attrs_higher: 0,
        };

        descriptor.base = base as u16;
        descriptor.middle_base = (base >> 16) as u8;
        descriptor.limit = limit as u16;
        descriptor.attrs_higher = ((limit >> 16) as u16) & 0xF; // 0xF = first 4 bits
        descriptor.attrs_higher |= (long_mode as u16) << (53 - 32 - 16);
        descriptor.attrs_higher |= (bit_32_mode as u16) << (54 - 32 - 16);
        descriptor.attrs_higher |= (granularity as u16) << (55 - 32 - 16);

        descriptor
    }
}

impl NormalSegment<UserAccessByte> {
    pub const NULL: Self = NormalSegment::new(0, 0, false, false, false, UserAccessByte::NULL);

    pub const KERNEL_CODE: Self = Self::new(
        0,
        0xFFFFF,
        true,
        false,
        true,
        UserAccessByte::new(
            AccessByte::new(true, PrivilegeLevel::Ring0),
            true,
            true,
            false,
        ),
    );

    pub const KERNEL_DATA: Self = Self::new(
        0,
        0xFFFFF,
        false,
        true,
        true,
        UserAccessByte::new(
            AccessByte::new(true, PrivilegeLevel::Ring0),
            false,
            true,
            false,
        ),
    );

    pub const USER_CODE: Self = Self::new(
        0,
        0xFFFFF,
        true,
        false,
        true,
        UserAccessByte::new(
            AccessByte::new(true, PrivilegeLevel::Ring3),
            true,
            true,
            false,
        ),
    );

    pub const USER_DATA: Self = Self::new(
        0,
        0xFFFFF,
        false,
        true,
        true,
        UserAccessByte::new(
            AccessByte::new(true, PrivilegeLevel::Ring3),
            false,
            true,
            false,
        ),
    );

    pub const fn as_u64(&self) -> u64 {
        let copy = *self;
        unsafe { core::mem::transmute(copy) }
    }

    pub fn privilege(&self) -> PrivilegeLevel {
        self.access_byte.value.privilege()
    }
}

impl NormalSegment<SystemAccessByte> {
    pub const fn as_u64(&self) -> u64 {
        let copy = *self;
        unsafe { core::mem::transmute(copy) }
    }

    pub fn privilege(&self) -> PrivilegeLevel {
        self.access_byte.value.privilege()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LongSegment {
    flags: NormalSegment<SystemAccessByte>,
    higher_base: u32,
    _reserved: u32,
}

impl LongSegment {
    const fn new(
        base: u64,
        limit: u32,
        long_mode: bool,
        bit_32_mode: bool,
        granularity: bool,
        access: SystemAccessByte,
    ) -> Self {
        Self {
            flags: NormalSegment::new(
                base as u32,
                limit,
                long_mode,
                bit_32_mode,
                granularity,
                access,
            ),
            higher_base: (base >> 32) as u32,
            _reserved: 0,
        }
    }

    pub fn new_tss(tss: &'static TaskStateSegment) -> Self {
        let tss_ptr = tss as *const _ as u64;
        let access_byte = SystemAccessByte::new(
            AccessByte::new(true, PrivilegeLevel::Ring0),
            SystemAccessByteKind::TssAvailable,
        );
        Self::new(
            tss_ptr,
            (size_of::<TaskStateSegment>() - 1) as u32,
            false,
            false,
            false,
            access_byte,
        )
    }

    pub fn as_u128(&self) -> u128 {
        let copy = *self;
        unsafe { core::mem::transmute(copy) }
    }

    pub fn as_u64(&self) -> (u64, u64) {
        let value = self.as_u128();
        (value as u64, (value >> 64) as u64)
    }

    pub fn privilege(&self) -> PrivilegeLevel {
        self.flags.access_byte.value.privilege()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_kernel_code64_segment() {
        let segment = NormalSegment::KERNEL_CODE;
        let binary_value = segment.as_u64();
        assert_eq!(binary_value, 0xaf9a000000ffff);
    }

    #[test_case]
    fn test_kernel_data_segment() {
        let segment = NormalSegment::KERNEL_DATA;
        let binary_value = segment.as_u64();
        assert_eq!(binary_value, 0xcf92000000ffff);
    }

    #[test_case]
    fn test_user_code_segment() {
        let segment = NormalSegment::USER_CODE;
        let binary_value = segment.as_u64();
        assert_eq!(binary_value, 0xaffa000000ffff);
    }

    #[test_case]
    fn test_user_data_segment() {
        let segment = NormalSegment::USER_DATA;
        let binary_value = segment.as_u64();
        assert_eq!(binary_value, 0xcff2000000ffff);
    }

    #[test_case]
    fn test_null_segment() {
        let segment = NormalSegment::NULL;
        let binary_value = segment.as_u64();
        assert_eq!(binary_value, 0);
    }
}
