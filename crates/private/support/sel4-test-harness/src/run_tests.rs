//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use alloc::string::String;
use core::fmt;
use core::panic::AssertUnwindSafe;

use sel4_panicking::catch_unwind;
use sel4_panicking_env::{debug_print, debug_println};

use crate::{
    config::types::*,
    for_generated_code::{Runnable, ShouldPanic, TestDescAndFn, TestFn},
};

pub fn run_tests_with_config(config: &Config, tests: &[&TestDescAndFn]) {
    debug_println!();
    debug_println!("running {} tests", tests.len());

    let mut num_passed = 0;
    let mut num_failed = 0;
    let mut num_ignored = 0;

    for test in tests.into_iter().map(make_owned_test) {
        debug_print!("test {} ... ", test.desc.name);
        let ignore = if test.desc.ignore {
            config.run_ignored == RunIgnored::No
        } else {
            config.run_ignored == RunIgnored::Only
        };
        if ignore {
            num_ignored += 1;
            debug_print!("... ignored");
            if let Some(message) = test.desc.ignore_message {
                debug_print!(", {message}");
            }
            debug_println!("");
        } else {
            let result = match test.testfn.into_runnable() {
                Runnable::Test(runnable) => wrap_run(test.desc.should_panic, || runnable.run()),
            };
            match result {
                TestResult::Ok => num_passed += 1,
                TestResult::Failed => num_failed += 1,
            }
            debug_println!("... {result}");
        }
    }

    assert_eq!(tests.len(), num_passed + num_failed + num_ignored);

    let result = TestResult::from(num_failed == 0);

    debug_println!();
    debug_println!(
        "test result: {result}. {num_passed} passed; {num_failed} failed; {num_ignored} ignored",
    );
    debug_println!();

    match result {
        TestResult::Ok => debug_println!("TEST_PASS"),
        TestResult::Failed => debug_println!("TEST_FAIL"),
    }
}

fn make_owned_test(test: &&TestDescAndFn) -> TestDescAndFn {
    match test.testfn {
        TestFn::StaticTestFn(f) => TestDescAndFn {
            testfn: TestFn::StaticTestFn(f),
            desc: test.desc.clone(),
        },
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum TestResult {
    Ok,
    Failed,
}

impl TestResult {
    #[allow(dead_code)]
    fn ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    #[allow(dead_code)]
    fn failed(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

impl From<bool> for TestResult {
    fn from(passed: bool) -> Self {
        match passed {
            true => Self::Ok,
            false => Self::Failed,
        }
    }
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "ok"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

fn wrap_run(should_panic: ShouldPanic, f: impl FnOnce() -> Result<(), String>) -> TestResult {
    match catch_unwind(AssertUnwindSafe(f)) {
        Err(_) => TestResult::from(should_panic.should_panic()),
        Ok(Ok(())) => TestResult::from(!should_panic.should_panic()),
        Ok(Err(msg)) => {
            debug_println!();
            debug_println!("{}", msg);
            debug_println!();
            TestResult::Failed
        }
    }
}
