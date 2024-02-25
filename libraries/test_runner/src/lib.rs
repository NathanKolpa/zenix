#![feature(custom_test_frameworks)]
#![test_runner(crate::runner)]

use std::io::{stdout, Write};

const TEST_NAME_ALIGN_TO: usize = 100;

pub trait TestCase {
    fn run(&self, test_number: usize, test_count: usize);
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self, test_number: usize, test_count: usize) {
        let test_name = core::any::type_name::<T>();
        let padding = TEST_NAME_ALIGN_TO.saturating_sub(test_name.len());

        print!(
            "  ({test_number:0<2}/{test_count:0<2}) => {test_name}...{: <1$}",
            "", padding
        );
        stdout().flush().unwrap();
        self();
        println!("[ok]");
    }
}

pub fn runner(tests: &[&dyn TestCase]) {
    println!("Running {} unit tests in host environment:", tests.len());

    for (i, test) in tests.iter().enumerate() {
        test.run(i + 1, tests.len());
    }

    println!("All unit tests completed successfully, shutting down...");
}
