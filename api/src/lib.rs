// #![feature(wasm_import_memory)]
// #[linkage = "--import-memory"]

pub mod host;

use std::{cell::RefCell, collections::BTreeMap, fmt, io, marker::PhantomData, ptr::NonNull};

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub enum CLType {
    Bool,
    String,
    Unit,
    Any,
}

pub trait CLTyped {
    fn cl_type() -> CLType;
}

impl CLTyped for String {
    fn cl_type() -> CLType {
        CLType::String
    }
}
impl CLTyped for bool {
    fn cl_type() -> CLType {
        CLType::Bool
    }
}

impl CLTyped for () {
    fn cl_type() -> CLType {
        CLType::Unit
    }
}

#[cfg(not(target_arch = "wasm32"))]
use serde::{Serialize, Deserialize};
#[derive(Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct SchemaArgument {
    pub name: &'static str,
    pub ty: CLType,
}

#[derive(Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]

pub struct SchemaEntryPoint {
    pub name: &'static str,
    pub arguments: Vec<SchemaArgument>,
}

#[derive(Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct SchemaData {
    pub name: &'static str,
    pub ty: CLType,
}

#[derive(Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct Schema {
    pub name: &'static str,
    pub data: Vec<SchemaData>,
    pub entry_points: Vec<SchemaEntryPoint>,
}

#[derive(Debug)]
pub struct Value<T> {
//  type Type = T;

    name: &'static str,
    key_space: u64,
    _marker: PhantomData<T>,
}
impl<T: CLTyped> CLTyped for Value<T> {
    fn cl_type() -> CLType {
        T::cl_type()
    }
}
pub fn reserve_vec_space(vec: &mut Vec<u8>, size: usize) -> NonNull<u8> {
    *vec = Vec::with_capacity(size);
    unsafe {
        vec.set_len(size);
    }
    NonNull::new(vec.as_mut_ptr()).expect("non null ptr")
}

impl<T> Value<T> {
    pub fn new(name: &'static str, key_space: u64) -> Self {
        Self {
            name,
            key_space,
            _marker: PhantomData,
        }
    }
}

impl<T: BorshSerialize> Value<T> {
    pub fn set(&mut self, value: T) -> io::Result<()> {
        // let mut value = Vec::new();
        // value.serialize(&mut value)?;
        let v = borsh::to_vec(&value)?;
        host::write(self.key_space, self.name.as_bytes(), 0, &v)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, "todo"))?;
        Ok(())
    }
}
impl<T: BorshDeserialize> Value<T> {
    pub fn get(&self) -> io::Result<Option<T>> {
        let mut read = None;//Vec::new();
        host::read(self.key_space, self.name.as_bytes(), |size| {
            *(&mut read) = Some(Vec::new());
            reserve_vec_space(read.as_mut().unwrap(), size)
        })
        .map_err(|error| io::Error::new(io::ErrorKind::Other, "todo"))?;
        match read {
            Some(mut read) => {
                let value = T::deserialize(&mut read.as_slice())?;
                Ok(Some(value))
            },
            None => Ok(None),
        }
    }
}

pub trait Contract {
    fn new() -> Self;
    fn name() -> &'static str;
    fn schema() -> Schema;
}

#[derive(Debug)]
pub enum Access {
    Private,
    Public,
}

#[derive(Debug)]
pub struct EntryPoint<'a, F: Fn()> {
    pub name: &'a str,
    pub params: &'a [(&'a str, CLType)],
    // pub access: Access,
    // fptr: extern "C" fn() -> (),
    pub func: F,
}

#[derive(Debug)]
pub enum ApiError {
    Error1,
    Error2,
    MissingArgument,
    Io(io::Error),
}

thread_local! {
    pub static DISPATCHER: RefCell<BTreeMap<String, extern "C" fn()>> = Default::default();
    pub static ARGS: RefCell<BTreeMap<String, Vec<u8>>> = Default::default();
}

#[no_mangle]
pub unsafe fn register_func(name: &str, f: extern "C" fn() -> ()) {
    println!("registering function {}", name);
    DISPATCHER.with(|foo| foo.borrow_mut().insert(name.to_string(), f));
}

pub fn register_entrypoint<'a, F: fmt::Debug + Fn()>(entrypoint: EntryPoint<'a, F>) {
    dbg!(entrypoint);
    // dbg!(&entrypoint);
    // unsafe {
    //     register_func(entrypoint.name, entrypoint.fptr);
    // }
}

pub fn get_named_arg<T: BorshDeserialize>(name: &str) -> Result<T, ApiError> {
    let arg_bytes = ARGS
        .with(|args| args.borrow().get(name).cloned())
        .ok_or(ApiError::MissingArgument)?;
    let mut slice = arg_bytes.as_slice();

    let deser: T = BorshDeserialize::deserialize(&mut slice).map_err(ApiError::Io)?;
    Ok(deser)
}
