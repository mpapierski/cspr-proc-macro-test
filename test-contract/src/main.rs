#![cfg_attr(target_arch = "wasm32", no_main)]
#![cfg_attr(target_arch = "wasm32", no_std)]

#[macro_use]
extern crate alloc;
use core::ptr::NonNull;

use alloc::vec::Vec;
use alloc::string::String;
use api::Value;
use macros::{casper, Contract};

#[derive(Contract, Debug)]
struct Flipper {
    flag: Value<bool>,
}

// #[casper(contract)]
// #[casper(entry_point)]
#[casper(entry_points)]
impl Flipper {
    pub fn flip(&mut self, argument1: bool, argument2: String) {
        let mut value = self.flag.get().unwrap().unwrap_or_default();
        value = !value;
        self.flag.set(value).unwrap();
    }

    pub fn flag_value(&self) -> bool {
        self.flag.get().unwrap().unwrap_or_default()
    }
}

// extern "C" fn flip(arg1: *const Slice, arg2: *const Slice);

mod exports {

    use alloc::string::String;
    use alloc::vec::Vec;
    use api::{
        host::{self, EntryPoint, Param, Slice},
        reserve_vec_space,
    };
    use macros::casper;

    // use crate::reserve_vec_space;

    const KEY_SPACE_DEFAULT: u64 = 0;
    const TAG_BYTES: u64 = 0;

    fn mangled_entry_point(param_1: *const Slice, param_2: *const Slice) {
        host::revert(123);
    }

    #[casper(export)]
    pub fn call(arg1: &[u8], arg2: &[u8], arg3: &[u8]) {
        host::print(&format!(
            "arg1={:?} arg2={:?} arg3={:?}",
            core::str::from_utf8(arg1),
            core::str::from_utf8(arg2),
            core::str::from_utf8(arg3)
        ));

        let mut read1 = Vec::new();

        let _non_existing_entry = host::read(KEY_SPACE_DEFAULT, b"hello", |size| {
            host::print(&format!("first cb alloc cb with size={size}"));
            reserve_vec_space(&mut read1, size)
            // static_buffer.as_mut_ptr()
        })
        .expect("should read");
        // host::print(&format!("non_existing_entry={:?}", non_existing_entry));
        host::write(KEY_SPACE_DEFAULT, b"hello", TAG_BYTES, b"Hello, world!").unwrap();

        let mut read2 = Vec::new();
        let existing_entry = host::read(KEY_SPACE_DEFAULT, b"hello", |size| {
            host::print(&format!("second cb alloc cb with size={size}"));
            reserve_vec_space(&mut read2, size)
        })
        .expect("should read")
        .expect("should have entry");
        host::print(&format!("existing_entry={:?}", existing_entry));
        let msg = String::from_utf8(read2).unwrap();
        host::print(&format!("existing_entry={:?}", msg));

        host::write(KEY_SPACE_DEFAULT, b"read back", TAG_BYTES, msg.as_bytes()).unwrap();

        const NAME: &str = "test_fptr";

        const PARAM_1: &str = "param_1";
        const PARAM_2: &str = "param_2";

        let params = [
            Param {
                name_ptr: PARAM_1.as_ptr(),
                name_len: PARAM_1.len(),
                ty: 0,
            },
            Param {
                name_ptr: PARAM_2.as_ptr(),
                name_len: PARAM_2.len(),
                ty: 0,
            },
        ];

        extern "C" fn mangled_entry_point_wrapper(param_1: *const Slice, param_2: *const Slice) {
            mangled_entry_point(param_1, param_2)
        }

        let fptr_void: *const core::ffi::c_void = mangled_entry_point_wrapper as _;

        let entry_point = EntryPoint {
            name_ptr: NAME.as_ptr(),
            name_len: NAME.len(),
            params_ptr: params.as_ptr(),
            params_size: params.len(),
            fptr: fptr_void,
        };
        host::print(&format!("{entry_point:?}"));
        // host::revert(123);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    todo!()
}
#[cfg(test)]
mod tests {

    use api::Contract;

    use super::*;
    #[test]
    fn test() {
        exports::call(b"hello", b"world", b"asdf");
    }

    #[test]
    fn compile_time_schema() {
        let schema = Flipper::schema();
        // dbg!(&schema);
        assert_eq!(schema.name, "Flipper");
        assert_eq!(schema.entry_points[0].name, "flip");

        let s = serde_json::to_string_pretty(&schema).expect("foo");
        println!("{s}");

        let mut flipper = Flipper::new();
        assert_eq!(Flipper::name(), "Flipper");
        // dbg!(&flipper);
        // flipper.flip(false, "Hello, world!".to_string());
        // let value = flipper.flag_value();
        // assert_eq!(value, true);
    }
}
