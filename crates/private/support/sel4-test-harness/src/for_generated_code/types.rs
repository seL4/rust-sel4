//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Rust project contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use alloc::borrow::Cow;
use alloc::fmt;
use alloc::string::String;

use crate::short_backtrace::__rust_begin_short_backtrace;

pub use NamePadding::*;
pub use TestFn::*;
pub use TestName::*;

/// Whether test is expected to panic or not
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShouldPanic {
    No,
    Yes,
    YesWithMessage(&'static str),
}

impl ShouldPanic {
    pub(crate) fn should_panic(&self) -> bool {
        !matches!(self, Self::No)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TestType {
    UnitTest,
    IntegrationTest,
    DocTest,
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NamePadding {
    PadNone,
    PadOnRight,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TestName {
    StaticTestName(&'static str),
    DynTestName(String),
    AlignedTestName(Cow<'static, str>, NamePadding),
}

impl TestName {
    pub fn as_slice(&self) -> &str {
        match *self {
            StaticTestName(s) => s,
            DynTestName(ref s) => s,
            AlignedTestName(ref s, _) => s,
        }
    }

    pub fn padding(&self) -> NamePadding {
        match self {
            &AlignedTestName(_, p) => p,
            _ => PadNone,
        }
    }

    pub fn with_padding(&self, padding: NamePadding) -> TestName {
        let name = match *self {
            TestName::StaticTestName(name) => Cow::Borrowed(name),
            TestName::DynTestName(ref name) => Cow::Owned(name.clone()),
            TestName::AlignedTestName(ref name, _) => name.clone(),
        };

        TestName::AlignedTestName(name, padding)
    }
}

impl fmt::Display for TestName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_slice(), f)
    }
}

pub enum TestFn {
    StaticTestFn(fn() -> Result<(), String>),
}

impl TestFn {
    pub fn padding(&self) -> NamePadding {
        match *self {
            StaticTestFn(..) => PadNone,
        }
    }

    pub(crate) fn into_runnable(self) -> Runnable {
        match self {
            StaticTestFn(f) => Runnable::Test(RunnableTest::Static(f)),
        }
    }
}

impl fmt::Debug for TestFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            StaticTestFn(..) => "StaticTestFn(..)",
        })
    }
}

pub(crate) enum Runnable {
    Test(RunnableTest),
}

pub(crate) enum RunnableTest {
    Static(fn() -> Result<(), String>),
}

impl RunnableTest {
    pub(crate) fn run(self) -> Result<(), String> {
        match self {
            RunnableTest::Static(f) => __rust_begin_short_backtrace(f),
        }
    }
}

// A unique integer associated with each test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TestId(pub usize);

// The definition of a single test. A test runner will run a list of
// these.
#[derive(Clone, Debug)]
pub struct TestDesc {
    pub name: TestName,
    pub ignore: bool,
    pub ignore_message: Option<&'static str>,
    pub source_file: &'static str,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub should_panic: ShouldPanic,
    pub compile_fail: bool,
    pub no_run: bool,
    pub test_type: TestType,
}

impl TestDesc {
    pub fn padded_name(&self, column_count: usize, align: NamePadding) -> String {
        let mut name = String::from(self.name.as_slice());
        let fill = column_count.saturating_sub(name.len());
        let pad = " ".repeat(fill);
        match align {
            PadNone => name,
            PadOnRight => {
                name.push_str(&pad);
                name
            }
        }
    }

    /// Returns None for ignored test or tests that are just run, otherwise returns a description of the type of test.
    /// Descriptions include "should panic", "compile fail" and "compile".
    pub fn test_mode(&self) -> Option<&'static str> {
        if self.ignore {
            return None;
        }
        match self.should_panic {
            ShouldPanic::Yes | ShouldPanic::YesWithMessage(_) => {
                return Some("should panic");
            }
            ShouldPanic::No => {}
        }
        if self.compile_fail {
            return Some("compile fail");
        }
        if self.no_run {
            return Some("compile");
        }
        None
    }
}

#[derive(Debug)]
pub struct TestDescAndFn {
    pub desc: TestDesc,
    pub testfn: TestFn,
}
