#[derive(Debug)]
pub enum Error {
    Foo,
    Bar,
}

// pub trait Host {
//     fn read(&mut self, key_space: u64, key: &[u8], size: *mut u64, tag: &mut u64) -> Result<Vec<u8>, Error>;
//     fn write(&mut self, key_space: u64, key: &[u8], value_tag: u64, value: &[u8]) -> Result<(), Error>;
// }

#[cfg(not(target_arch = "wasm32"))]
mod native_support {
    use std::{
        cell::RefCell,
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use once_cell::sync::Lazy;

    // pub(crate) type ExportFn = dyn Fn(&[&[u8]]);

    // static EXPORTS: Lazy<Arc<Mutex<HashMap<String, Box<dyn Fn(&[&[u8]])>>>>> = Lazy::new(Default::default);
}

#[derive(Debug)]
pub struct Entry {
    data: Bytes,
    tag: u64,
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use bytes::Bytes;
    use core::slice;
    use std::{
        ffi::c_void,
        mem::{self, MaybeUninit},
        ptr::{self, NonNull},
    };

    use super::{Entry, Error};
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
                data: Bytes::from(data),
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

    // pub(crate) fn clone_db() -> LocalKV {
    //     DB.with(|kv| kv.borrow().clone())
    // }

    // pub(crate) fn commit_db(db: LocalKV) {
    //     DB.with(|kv| {*kv.borrow_mut() = db;})
    // }

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
    pub fn read(key_space: u64, key: &[u8]) -> Result<Option<Entry>, Error> {
        let value = DB.with(|db| db.borrow().db.get(&key_space)?.get(key).cloned());
        match value {
            Some(tagged_value) => Ok(Some(Entry {
                data: tagged_value.value.clone(),
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

use bytes::Bytes;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{print, read, revert, write};
#[cfg(target_arch = "wasm32")]
pub use wasm::{print, read, revert, write, Slice};
