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

pub fn test_runner(tests: &[&dyn TestCase]) {
    for test in tests {
        test.run();
    }
}
