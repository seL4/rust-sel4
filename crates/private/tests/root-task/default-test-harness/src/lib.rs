//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

#[cfg(test)]
mod test {
    #[test]
    fn foo() {}

    #[test]
    #[should_panic]
    fn bar() {
        assert!(false);
    }
}

mod m {
    #[cfg(test)]
    mod test {
        #[test]
        fn foo() {}

        #[ignore = "a reason"]
        #[test]
        fn bar() {
            assert!(false);
        }
    }
}

// cargo rustc $h --target aarch64-sel4 -p tests-root-task-default-test-harness --profile=check -- --test -Zunpretty=expanded
// cargo rustc $h --target riscv64imac-sel4 -p tests-root-task-default-test-harness --profile=check -- --test -Zunpretty=expanded
