extern crate proc_macro;
use std::str::FromStr;

use api::{get_named_arg, EntryPoint};
use paste::paste;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Item, ItemFn};

#[proc_macro_attribute]
pub fn casper(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // eprintln!("{attr:#?}");
    let mut tokens = Vec::new();
    let func = parse_macro_input!(item as ItemFn);

    for attr in attrs {
        if attr.to_string() == "call" {
            let func_name = &func.sig.ident;
            let body = &func.block;
            // let func = func;

            // let mangled = format!("mod mangled_{func_name} {{ {func} }}");
            // let mangled = TokenStream::from_str(&mangled).unwrap();

            let token = quote! {
                #[no_mangle]
                pub extern "C" fn call() {
                    #func;
                    #func_name()
                    // wasm_export_ #func_name::#func_name();
                }
            };
            tokens.push(token);
        }
    }
    // println!("{}", &tokens.get(0).unwrap());
    quote! {
        #(#tokens)*;
    }
    .into()
}

#[proc_macro_attribute]
pub fn entry_point(attr: TokenStream, item: TokenStream) -> TokenStream {
    eprintln!("{attr:#?}");
    let item2 = item.clone();
    let func = parse_macro_input!(item2 as ItemFn);
    eprintln!("{:#?}", func);

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
