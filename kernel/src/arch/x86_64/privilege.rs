/// From the [osdev wiki](https://wiki.osdev.org/Security):
/// > Rings offer a protection layer for programs.
/// > They allow certain levels of resource access to processes.
/// > This is good, because it keeps bad programs from messing things up.
/// > There are, however, several downsides: The more CPU rings you use, the more the OS is tied to the architecture.
///>  You can, however, have several architectures each with it's own ring switching code.
/// > Another issue with this is that you OS must have a TSS set up and several other features,
/// making ring switching much more difficult than just running all programs in kernel mode.
/// > There are a total of 4 rings in most common architectures.
/// However, many architectures have only two rings (e.g. x86_64), corresponding to ring 0 and 3 in this description.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PrivilegeLevel {
    /// The privilege level the kernel runs in.
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,

    /// The privilege level user code runs in.
    Ring3 = 3,
}

impl From<u8> for PrivilegeLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            _ => panic!("Invalid privilege level"),
        }
    }
}
