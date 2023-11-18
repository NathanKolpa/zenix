use core::panic::PanicInfo;

pub trait TestCase {
    fn run(&self);
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self) {
        self();
    }
}

pub fn runner(tests: &[&dyn TestCase]) {
    for test in tests {
        test.run();
    }
}
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
