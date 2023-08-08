#![cfg_attr(target_arch = "wasm32", no_main)]
#![cfg_attr(target_arch = "wasm32", no_std)]

// extern crate ""
// use alloc::collections::BTreeMap;
#[macro_use]
extern crate alloc;

use core::{
    mem,
    ptr::{self, NonNull},
};

use alloc::slice;
use api::host;
// use api::{dispatch, host::{self}, EntryPoint};
// use api:
use macros::casper;

// pub(crate) mod foo {
//     pub(crate) mod bar {
//         pub(crate) mod baz {
//             use macros::entry_point;

//             #[entry_point]
//             pub(crate) fn mangled_entry_point(argument_1: String) -> Result<(), api::ApiError> {
//                 println!("hello from mangled entry point: {}", argument_1);
//                 Ok(())
//             }
//         }
//     }
// }

// macro_rules! entry_points {
//     ($f:expr) => {{
//         extern "C" fn wrapper() {
//             ($f.fptr)();
//         }

//         unsafe {
//             register_func(stringify!($f), wrapper);
//         }
//     }};
//     ($f:expr => $renamed:expr) => {{
//         extern "C" fn wrapper() {
//             $f();
//         }
//         unsafe {
//             api::register_func($renamed, wrapper);
//         }
//     }};
// }

// #[casper(call)]
// const KEY: [u8; 32] = ""
const KEY_SPACE_DEFAULT: u64 = 0;
const TAG_BYTES: u64 = 0;

// #[casper(export)]
// // #[no_mangle]
// fn call(argc: u64, slices: *const Slice) {
//     host::print(&format!("argc={argc}"));
//     for i in 0..argc {
//         let slice = unsafe { slices.offset(i as _).read() };
//         let real_slice = unsafe { slice::from_raw_parts(slice.ptr, slice.size) };
//         let s = core::str::from_utf8(real_slice).unwrap();
//         host::print(&format!("slice[{i}]={s:?} ({slice:?})"));
//     }

//     // host::revert(1234);
// }

mod exports {
    use api::host;
    use macros::casper;

    use crate::{KEY_SPACE_DEFAULT, TAG_BYTES};

    #[casper(export)]
    pub fn call(arg1: &[u8], arg2: &[u8], arg3: &[u8]) {
        host::print(&format!(
            "arg1={:?} arg2={:?} arg3={:?}",
            core::str::from_utf8(arg1),
            core::str::from_utf8(arg2),
            core::str::from_utf8(arg3)
        ));

        let non_existing_entry = host::read(KEY_SPACE_DEFAULT, b"hello").expect("should read");
        host::print(&format!("non_existing_entry={:?}", non_existing_entry));
        host::write(KEY_SPACE_DEFAULT, b"hello", TAG_BYTES, b"world").unwrap();
        let existing_entry = host::read(KEY_SPACE_DEFAULT, b"hello").expect("should read");
        host::print(&format!("existing_entry={:?}", existing_entry));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        exports::call(b"hello", b"world", b"asdf");
    }
}
