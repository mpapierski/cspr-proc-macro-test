#[derive(Debug)]
pub enum Error {
    Foo,
    Bar,
}

// pub trait Host {
//     fn read(&mut self, key_space: u64, key: &[u8], size: *mut u64, tag: &mut u64) -> Result<Vec<u8>, Error>;
//     fn write(&mut self, key_space: u64, key: &[u8], value_tag: u64, value: &[u8]) -> Result<(), Error>;
// }

#[derive(Debug)]
pub struct Entry {
    data: Vec<u8>,
    tag: u64,
}

mod wasm {
    use std::{
        mem::{self, MaybeUninit},
        ptr::{NonNull, self}, ffi::c_void,
    };

    use super::{Entry, Error};

    #[derive(Debug)]
    #[repr(C)]
    pub struct ReadInfo {
        data: *mut u8,
        /// Size in bytes.
        size: u64,
        /// Value tag.
        tag: u64,
    }

    extern "C" {
        pub fn casper_read(
            key_space: u64,
            key_ptr: *const u8,
            key_size: usize,
            info: *mut ReadInfo,
        ) -> i32;
        pub fn casper_write(
            key_space: u64,
            key_ptr: *const u8,
            key_size: usize,
            value_tag: u64,
            value_ptr: *const u8,
            value_size: usize,
        ) -> i32;
        pub fn casper_print(msg_ptr: *const u8, msg_size: usize) -> i32;
        pub fn casper_revert(code: u32);
        // pub fn foo(slice: *const Slice);
    }

    pub fn print(msg: &str) {
        let _res = unsafe { casper_print(msg.as_ptr(), msg.len()) };
    }

    pub fn revert(code: u32) -> ! {
        unsafe { casper_revert(code) };
        unreachable!()
    }

    pub fn read(key_space: u64, key: &[u8]) -> Result<Option<Entry>, Error> {
        let mut info = MaybeUninit::uninit();

        let ret = unsafe { casper_read(key_space, key.as_ptr(), key.len(), info.as_mut_ptr()) };

        if ret == 0 {
            let info = unsafe { info.assume_init() };
            let data = unsafe { Vec::from_raw_parts(info.data, info.size as _, info.size as _) };
            Ok(Some(Entry {
                data,
                tag: info.tag,
            }))
        } else if ret == 1 {
            Ok(None)
        } else {
            Err(Error::Foo)
        }
    }

    pub fn write(key_space: u64, key: &[u8], value_tag: u64, value: &[u8]) -> Result<(), Error> {
        let _ret = unsafe {
            casper_write(
                key_space,
                key.as_ptr(),
                key.len(),
                value_tag,
                value.as_ptr(),
                value.len(),
            )
        };
        Ok(())
    }

    #[no_mangle]
    pub extern "C" fn alloc(len: usize) -> *mut u8 {
        // Create a new mutable buffer with capacity `len`
        let mut buf = Vec::with_capacity(len);
        // Take a mutable pointer to the buffer
        let ptr = buf.as_mut_ptr();
        // Prevent the buffer from being deallocated when it goes out of scope
        mem::forget(buf);
        // Return the pointer so the runtime can write data at this offset
        ptr
    }
}

pub use wasm::{print, read, revert, write};

// #[cfg(not(target_arch = "wasm32"))]
