#[derive(Debug)]
pub enum Error {
    Foo,
    Bar,
}

#[derive(Debug)]
pub struct Entry {
    pub tag: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct Slice {
    ptr: *const u8,
    size: usize,
}

impl Slice {
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.size) }
    }
}
pub struct Param {
    pub name_ptr: *const u8,
    pub name_len: usize,
    pub ty: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct EntryPoint {
    pub name_ptr: *const u8,
    pub name_len: usize,

    pub params_ptr: *const Param, // pointer of pointers (preferred 'static lifetime)
    pub params_size: usize,

    pub fptr: *const c_void, // extern "C" fn(A1) -> (),
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use bytes::Bytes;

    use std::{
        alloc::{self, Layout},
        ffi::c_void,
        mem::{self, MaybeUninit},
        ptr::{self, NonNull},
    };

    use super::{Entry, Error};

    #[derive(Debug)]
    #[repr(C)]
    pub struct ReadInfo {
        data: *const u8,
        /// Size in bytes.
        size: usize,
        /// Value tag.
        tag: u64,
    }

    extern "C" {
        pub fn casper_read(
            key_space: u64,
            key_ptr: *const u8,
            key_size: usize,
            info: *mut ReadInfo,
            alloc: extern "C" fn(usize, *mut c_void) -> *const u8,
            alloc_ctx: *const c_void,
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

        // pub fn casper_add_contract_version(hash_ptr: *const u8, hash_len: usize, entry_points);
        // pub fn foo(slice: *const Slice);
    }

    pub fn print(msg: &str) {
        let _res = unsafe { casper_print(msg.as_ptr(), msg.len()) };
    }

    pub fn revert(code: u32) -> ! {
        unsafe { casper_revert(code) };
        unreachable!()
    }

    pub fn read_into<'a>(
        key_space: u64,
        key: &[u8],
        destination: &'a mut [u8],
    ) -> Option<&'a [u8]> {
        let mut what_size: Option<usize> = None;
        read(key_space, key, |size| {
            what_size = Some(size);
            NonNull::new(destination.as_mut_ptr()).unwrap()
        });

        let size = what_size?;

        Some(&destination[..size])
    }

    pub fn read<F: FnOnce(usize) -> NonNull<u8>>(
        key_space: u64,
        key: &[u8],
        f: F,
    ) -> Result<Option<Entry>, Error> {
        // let mut info = MaybeUninit::uninit();
        let mut info = ReadInfo {
            data: ptr::null(),
            size: 0,
            tag: 0,
        };

        extern "C" fn alloc_cb<F: FnOnce(usize) -> NonNull<u8>>(
            len: usize,
            ctx: *mut c_void,
        ) -> *const u8 {
            let opt_closure = ctx as *mut Option<F>;
            let mut ptr = unsafe { (*opt_closure).take().unwrap()(len) };
            unsafe { ptr.as_mut() }
        }

        let ctx = &Some(f) as *const _ as *mut c_void;

        let ret = unsafe {
            casper_read(
                key_space,
                key.as_ptr(),
                key.len(),
                &mut info as *mut ReadInfo,
                alloc_cb::<F>,
                ctx,
            )
        };

        if ret == 0 {
            Ok(Some(Entry { tag: info.tag }))
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

    #[no_mangle]
    pub extern "C" fn dealloc(ptr: *const c_void, len: usize) {
        let ptr: Vec<u8> = unsafe { Vec::from_raw_parts(ptr as _, len, len) };
        mem::drop(ptr);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::{
        borrow::{Borrow, BorrowMut},
        cell::RefCell,
        collections::BTreeMap,
    };

    use bytes::Bytes;

    use super::{Entry, Error};

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
    struct TaggedValue {
        tag: u64,
        value: Bytes,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
    struct BorrowedTaggedValue<'a> {
        tag: u64,
        value: &'a [u8],
    }
    type Container = BTreeMap<u64, BTreeMap<Bytes, TaggedValue>>;

    #[derive(Default, Clone)]
    pub(crate) struct LocalKV {
        db: Container,
    }

    // impl LocalKV {
    //     pub(crate) fn update(&mut self, db: LocalKV) {
    //         self.db = db.db
    //     }
    // }

    thread_local! {
        static DB: RefCell<LocalKV> = RefCell::new(LocalKV::default());
    }

    pub fn print(msg: &str) {
        println!("💻 {msg}");
    }
    pub fn write(key_space: u64, key: &[u8], value_tag: u64, value: &[u8]) -> Result<(), Error> {
        DB.with(|db| {
            // db.borrow_mut().db.insert(key_space, value: Bytes::copy_from_slice(key) }, TaggedValue { tag: value_tag, value: Bytes::copy_from_slice(value) })
            db.borrow_mut().db.entry(key_space).or_default().insert(
                Bytes::copy_from_slice(key),
                TaggedValue {
                    tag: value_tag,
                    value: Bytes::copy_from_slice(value),
                },
            );
        });
        Ok(())
    }
    pub fn read(
        key_space: u64,
        key: &[u8],
        func: impl FnOnce(usize) -> core::ptr::NonNull<u8>,
    ) -> Result<Option<Entry>, Error> {
        let value = DB.with(|db| db.borrow().db.get(&key_space)?.get(key).cloned());
        match value {
            Some(tagged_value) => Ok(Some(Entry {
                tag: tagged_value.tag,
            })),
            None => Ok(None),
        }
    }

    // pub fn dispatch<Args, R>(export: impl Fn(Args) -> R, args: Args) -> R {
    //     export(args)
    // }
    pub fn revert(code: u32) -> ! {
        panic!("revert with code {code}")
    }
}

use core::slice;
use std::ffi::c_void;

use bytes::Bytes;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{print, read, revert, write};
#[cfg(target_arch = "wasm32")]
pub use wasm::{print, read, revert, write};

// #[cfg(test)]
// mod tests {
//     use std::mem;

//     use super::*;
//     fn test_fptr(param_1: *const Slice, param_2: *const Slice) {

//     }

//     }
