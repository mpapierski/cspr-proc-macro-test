extern crate proc_macro;
use std::str::FromStr;

use api::{get_named_arg, EntryPoint};
use paste::paste;
use proc_macro::{Ident, TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input,
    token::{Crate, Pub},
    Data, DeriveInput, Item, ItemFn, VisRestricted, Visibility,
};

#[proc_macro_attribute]
pub fn casper(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    // eprintln!("{attrs:?}");

    let mut attrs_iter = attrs.into_iter().peekable();

    for attr in &mut attrs_iter {
        match attr {
            proc_macro::TokenTree::Ident(ident) if ident.to_string() == "export" => {
                let mut arg_slices = Vec::new();
                let mut arg_casts = Vec::new();
                let mut arg_calls = Vec::new();

                for input in &func.sig.inputs {
                    let typed = match input {
                        syn::FnArg::Receiver(_) => todo!(),
                        syn::FnArg::Typed(typed) => typed,
                    };
                    let name = match typed.pat.as_ref() {
                        syn::Pat::Ident(ident) => &ident.ident,
                        _ => todo!(),
                    };

                    // let name = input.n
                    let arg = quote! {
                        unsafe { core::ptr::NonNull::new_unchecked(#name).as_ref() }.as_slice()
                    };

                    arg_casts.push(arg);
                    let arg_slice = quote! {
                        #name: *mut api::host::Slice
                    };
                    arg_slices.push(arg_slice);

                    arg_calls.push(quote! {
                        name
                    })
                }

                // Ident::
                let mod_name = format_ident!("__casper__export_{func_name}");

                let token = quote! {
                    pub(crate) mod #mod_name {
                        use super::*;

                        #func
                    }

                    #[cfg(target_arch = "wasm32")]
                    #[no_mangle]
                    pub extern "C" fn #func_name( #(#arg_slices,)* ) {
                        #mod_name::#func_name(#(#arg_casts,)*);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    pub use #mod_name::#func_name;
                };
                // println!("{token:#}");
                return token.into();
            }
            _ => todo!(),
        }
    }
    todo!()
    // dbg!(&attrs);
    // // eprintln!("{attr:#?}");
    // let mut tokens = Vec::new();

    // for attr in attrs {
    //     if attr.to_string() == "call" {
    //         let func_name = &func.sig.ident;
    //         let body = &func.block;

    //         let token = quote! {
    //             #[no_mangle]
    //             pub extern "C" fn call() {
    //                 #func;
    //                 #func_name()
    //                 // wasm_export_ #func_name::#func_name();
    //             }
    //         };
    //         tokens.push(token);
    //     }
    // }
    // // println!("{}", &tokens.get(0).unwrap());
    // quote! {
    //     #(#tokens)*;
    // }
    // .into()
}

#[proc_macro_attribute]
pub fn entry_point(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let vis = &func.vis;
    let _sig = &func.sig;
    let func_name = &func.sig.ident;

    let block = &func.block;

    let mut handle_args = Vec::new();
    let mut params = Vec::new();

    for arg in &func.sig.inputs {
        let typed = match arg {
            syn::FnArg::Receiver(_) => todo!(),
            syn::FnArg::Typed(typed) => typed,
        };

        let name = match typed.pat.as_ref() {
            syn::Pat::Ident(ident) => &ident.ident,
            _ => todo!(),
        };

        let ty = &typed.ty;

        let tok = quote! {
            let #typed = api::get_named_arg(stringify!(#name)).expect("should get named arg");
        };
        handle_args.push(tok);

        let tok2 = quote! {
            (stringify!(#name), <#ty>::cl_type())
        };
        params.push(tok2);
    }

    // let len = params.len();

    let output = &func.sig.output;

    // let const_tok =

    let gen = quote! {
        // const paste!(#func_name, _ENTRY_POINT): &str = #func_name;

        #vis fn #func_name() {
            #(#handle_args)*;

            let closure = || #output {
                #block
            };

            let result = closure();

            // api::EntryPoint {
            //     name: #func_name,
            //     params: &[
            //         #(#params,)*
            //     ],
            //     func: closure,
            // }

            result.expect("should work")
        }
    };

    println!("{}", gen);

    // quote!(fn foo() {})
    // item
    gen.into()
}
//     // ItemFn
//     //  syn::parse(function).expect("should be function");
//     // let input2 = item.clone();
//     // let DeriveInput { ident, data, .. } = parse_macro_input!(input2);

//     // println!("attr: \"{}\"", attr.to_string());
//     // println!("item: \"{}\"", item.to_string());

//     // if let Data::
//     // println!("{}", data);
//     // if let
//     // item
// }
