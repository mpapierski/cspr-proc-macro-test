pub mod host;

use std::{cell::RefCell, collections::BTreeMap, fmt, io};

use borsh::BorshDeserialize;

#[derive(Debug)]
pub enum CLType {
    String,
    Unit,
}

pub trait CLTyped {
    fn cl_type() -> CLType;
}

impl CLTyped for String {
    fn cl_type() -> CLType {
        CLType::String
    }
}

impl CLTyped for () {
    fn cl_type() -> CLType {
        CLType::Unit
    }
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
