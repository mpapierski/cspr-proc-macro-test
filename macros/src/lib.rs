extern crate proc_macro;
use std::str::FromStr;

use api::{get_named_arg, EntryPoint};
use paste::paste;
use proc_macro::{Ident, TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input,
    token::{Crate, Pub},
    Data, DeriveInput, Item, ItemFn, ItemImpl, Type, VisRestricted, Visibility,
};

#[proc_macro_derive(Contract)]
pub fn derive_casper_contract(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    // let input = parse_macro_input!(input as DeriveInput);
 let contract = parse_macro_input!(input as DeriveInput);
                dbg!(&contract);
                // todo!("{contract:?}")
                let name = &contract.ident;
                let vis = &contract.vis;

                let data_struct = match &contract.data {
                    Data::Struct(s) => s,
                    Data::Enum(_) => todo!("Enum"),
                    Data::Union(_) => todo!("Union"),
                };

                let mut fields = Vec::new();
                let mut fields_for_schema = Vec::new();

                // let fields = data_struct.fields;
                let mut fields_for_new = Vec::new();
                for field in &data_struct.fields {
                    let name = &field.ident;
                    let ty = &field.ty;
                    // fields.push(field.clone());
                    fields.push(quote! {
                        #name: api::Value<#ty>
                    });

                    fields_for_schema.push(quote! {
                        api::SchemaData {
                            name: stringify!(#name),
                            ty: {
                                use api::CLTyped;
                                <#ty>::cl_type()
                            },
                        }
                    });

                    fields_for_new.push(quote! {
                        #name: api::Value::new(stringify!(#name), 0)
                    })
                }

                return quote! {
                    // #vis struct #name {
                    //     #(#fields,)*
                    // }
                    impl api::Contract for #name {
                        fn new() -> Self {
                            Self {
                                #(#fields_for_new,)*
                            }
                        }
                        fn name() -> &'static str {
                            stringify!(#name)
                        }

                        fn schema() -> api::Schema {
                            // todo!()
                            Self::__casper_schema()
                            // api::Schema {

                            // }
                        }


                    }

                    impl #name {
                        #[doc(hidden)]
                        fn __casper_data() -> Vec<api::SchemaData> {
                            vec! [
                                #(#fields_for_schema,)*
                            ]
                        }
                    }
                }
                .into();
}


#[proc_macro_attribute]
pub fn casper(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // eprintln!("{attrs:?}");

    let mut attrs_iter = attrs.into_iter().peekable();

    for attr in &mut attrs_iter {
        let item = item.clone();
        match attr {
            proc_macro::TokenTree::Ident(ident) if ident.to_string() == "entry_points" => {
                let entry_points = parse_macro_input!(item as ItemImpl);

                let name = match entry_points.self_ty.as_ref() {

                    Type::Path(ref path) => &path.path,

                    other => todo!("{other:?}"),
                };
                // todo!("{:?}", &entry_points);
                // let name = entry_points

                // }

                let mut defs = Vec::new();

                let mut names = Vec::new();
                for entry_point in &entry_points.items {
                    let func = match entry_point {
                        syn::ImplItem::Const(_) => todo!(),
                        syn::ImplItem::Fn(func) => {
                            let name = &func.sig.ident;
                            names.push(name);
                            func
                        }
                        syn::ImplItem::Type(_) => todo!(),
                        syn::ImplItem::Macro(_) => todo!(),
                        syn::ImplItem::Verbatim(_) => todo!(),
                        _ => todo!(),
                    };

                    let func_name = &func.sig.ident;

                        let mut args = Vec::new();

                    for input in &func.sig.inputs {
                        let typed = match input {
                            syn::FnArg::Receiver(_receiver) => continue,
                            syn::FnArg::Typed(typed) => typed,
                        };
                        // typed
                        let name = match &typed.pat.as_ref() {
                            syn::Pat::Const(_) => todo!("Const"),
                            syn::Pat::Ident(ident) => ident,
                            syn::Pat::Lit(_) => todo!("Lit"),
                            syn::Pat::Macro(_) => todo!("Macro"),
                            syn::Pat::Or(_) => todo!("Or"),
                            syn::Pat::Paren(_) => todo!("Paren"),
                            syn::Pat::Path(_) => todo!("Path"),
                            syn::Pat::Range(_) => todo!("Range"),
                            syn::Pat::Reference(_) => todo!("Reference"),
                            syn::Pat::Rest(_) => todo!("Rest"),
                            syn::Pat::Slice(_) => todo!("Slice"),
                            syn::Pat::Struct(_) => todo!("Struct"),
                            syn::Pat::Tuple(_) => todo!("Tuple"),
                            syn::Pat::TupleStruct(_) => todo!("TupleStruct"),
                            syn::Pat::Type(_) => todo!("Type"),
                            syn::Pat::Verbatim(_) => todo!("Verbatim"),
                            syn::Pat::Wild(_) => todo!("Wild"),
                            _ => todo!(),
                        };
                        let ty = &typed.ty;
                        args.push(quote! {
                            api::SchemaArgument {
                                name: stringify!(#name),
                                ty: {
                                    use api::CLTyped;
                                    <#ty>::cl_type()
                                },
                            }
                        });


                    }


                    // let mut args = Vec::new();
                    // for arg in &entry_point

                    defs.push(quote! {
                        api::SchemaEntryPoint {
                            name: stringify!(#func_name),
                            arguments: vec![ #(#args,)* ]
                        }
                    });
                }

                let res = quote! {
                    #entry_points

                    impl #name {
                        #[doc(hidden)]
                        fn __casper_schema() -> api::Schema {
                            let entry_points = vec![
                                #(#defs,)*
                                // EntryPonit
                            ];
                            let data = Self::__casper_data();
                            api::Schema {
                                name: stringify!(#name),
                                data,
                                entry_points,
                            }
                        }
                    }
                };
                eprintln!("{res}");
                return res.into();
            }
            proc_macro::TokenTree::Ident(ident) if ident.to_string() == "contract" => {

                todo!()
            }
            proc_macro::TokenTree::Ident(ident) if ident.to_string() == "export" => {
                let func = parse_macro_input!(item as ItemFn);
                let func_name = &func.sig.ident;

                let mut arg_slices = Vec::new();
                let mut arg_casts = Vec::new();
                let mut arg_calls = Vec::new();

                for input in &func.sig.inputs {
                    let name = match input {
                        syn::FnArg::Receiver(receiver) => {
                            todo!("{receiver:?}")
                        },
                        syn::FnArg::Typed(typed) => {
                            match typed.pat.as_ref() {
                                syn::Pat::Ident(ident) => &ident.ident,
                                _ => todo!(),
                            }
                        }
                    };
                    // let name = match typed.pat.as_ref() {
                    //     syn::Pat::Ident(ident) => &ident.ident,
                    //     _ => todo!(),
                    // };

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
