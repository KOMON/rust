// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Global storage for command line arguments
//!
//! The current incarnation of the Rust runtime expects for
//! the processes `argc` and `argv` arguments to be stored
//! in a globally-accessible location for use by the `os` module.
//!
//! Only valid to call on Linux. Mac and Windows use syscalls to
//! discover the command line arguments.
//!
//! FIXME #7756: Would be nice for this to not exist.

use core::prelude::*;
use collections::vec::Vec;

/// One-time global initialization.
pub unsafe fn init(argc: int, argv: *const *const u8) { imp::init(argc, argv) }

/// One-time global cleanup.
pub unsafe fn cleanup() { imp::cleanup() }

/// Take the global arguments from global storage.
pub fn take() -> Option<Vec<Vec<u8>>> { imp::take() }

/// Give the global arguments to global storage.
///
/// It is an error if the arguments already exist.
pub fn put(args: Vec<Vec<u8>>) { imp::put(args) }

/// Make a clone of the global arguments.
pub fn clone() -> Option<Vec<Vec<u8>>> { imp::clone() }

#[cfg(any(target_os = "linux",
          target_os = "android",
          target_os = "freebsd",
          target_os = "dragonfly"))]
mod imp {
    use core::prelude::*;

    use alloc::boxed::Box;
    use collections::vec::Vec;
    use collections::string::String;
    use core::mem;

    use mutex::{StaticNativeMutex, NATIVE_MUTEX_INIT};

    static mut GLOBAL_ARGS_PTR: uint = 0;
    static LOCK: StaticNativeMutex = NATIVE_MUTEX_INIT;

    pub unsafe fn init(argc: int, argv: *const *const u8) {
        let args = load_argc_and_argv(argc, argv);
        put(args);
    }

    pub unsafe fn cleanup() {
        rtassert!(take().is_some());
        LOCK.destroy();
    }

    pub fn take() -> Option<Vec<Vec<u8>>> {
        with_lock(|| unsafe {
            let ptr = get_global_ptr();
            let val = mem::replace(&mut *ptr, None);
            val.as_ref().map(|s: &Box<Vec<Vec<u8>>>| (**s).clone())
        })
    }

    pub fn put(args: Vec<Vec<u8>>) {
        with_lock(|| unsafe {
            let ptr = get_global_ptr();
            rtassert!((*ptr).is_none());
            (*ptr) = Some(box args.clone());
        })
    }

    pub fn clone() -> Option<Vec<Vec<u8>>> {
        with_lock(|| unsafe {
            let ptr = get_global_ptr();
            (*ptr).as_ref().map(|s: &Box<Vec<Vec<u8>>>| (**s).clone())
        })
    }

    fn with_lock<T, F>(f: F) -> T where F: FnOnce() -> T {
        unsafe {
            let _guard = LOCK.lock();
            f()
        }
    }

    fn get_global_ptr() -> *mut Option<Box<Vec<Vec<u8>>>> {
        unsafe { mem::transmute(&GLOBAL_ARGS_PTR) }
    }

    unsafe fn load_argc_and_argv(argc: int, argv: *const *const u8) -> Vec<Vec<u8>> {
        Vec::from_fn(argc as uint, |i| {
            String::from_raw_buf(*argv.offset(i as int)).into_bytes()
        })
    }

    #[cfg(test)]
    mod tests {
        use std::prelude::*;
        use std::finally::Finally;

        use super::*;

        #[test]
        fn smoke_test() {
            // Preserve the actual global state.
            let saved_value = take();

            let expected = vec![
                b"happy".to_vec(),
                b"today?".to_vec(),
            ];

            put(expected.clone());
            assert!(clone() == Some(expected.clone()));
            assert!(take() == Some(expected.clone()));
            assert!(take() == None);

            (|&mut:| {
            }).finally(|| {
                // Restore the actual global state.
                match saved_value {
                    Some(ref args) => put(args.clone()),
                    None => ()
                }
            })
        }
    }
}

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "windows"))]
mod imp {
    use core::prelude::*;
    use collections::vec::Vec;

    pub unsafe fn init(_argc: int, _argv: *const *const u8) {
    }

    pub fn cleanup() {
    }

    pub fn take() -> Option<Vec<Vec<u8>>> {
        panic!()
    }

    pub fn put(_args: Vec<Vec<u8>>) {
        panic!()
    }

    pub fn clone() -> Option<Vec<Vec<u8>>> {
        panic!()
    }
}
