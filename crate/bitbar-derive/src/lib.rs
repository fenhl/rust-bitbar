//! Proc macros for the `bitbar` crate.

#![deny(
    missing_docs,
    rust_2018_idioms, // this lint is actually about idioms that are *outdated* in Rust 2018
    unused,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    warnings,
)]

use {
    itertools::Itertools as _,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::{
        quote,
        quote_spanned,
    },
    syn::{
        *,
        punctuated::Punctuated,
        spanned::Spanned as _,
    },
};

/// Registers a subcommand that you can run from a menu item's `command`.
///
/// Commands may take any number of parameters implementing `FromStr` (with errors implementing `Display`) and `ToString`, and should return `Result<(), Error>`, where `Error` is any type that implements `Display`. If a command errors, `bitbar` will attempt to send a macOS notification containing the error message.
///
/// Alternatively, use this arrtibute as `#[command(varargs)]` and define the command function with a single parameter of type `Vec<String>`.
///
/// The `command` attribute generates a function that can be called with arguments of references to the original parameter types to obtain a `std::io::Result<Params>`. If the command has more than 5 parameters or is declared with `#[command(varargs)]`, the function takes an additional first parameter of type `SwiftBar`.
///
/// The function must also be registered via `#[bitbar::main(commands(...))]`.
#[proc_macro_attribute]
pub fn command(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, Token![,]>::parse_terminated);
    let varargs = match args.into_iter().at_most_one() {
        Ok(None) => false,
        Ok(Some(arg)) if arg.path().is_ident("varargs") => true,
        _ => return quote!(compile_error!("unexpected bitbar::command arguments");).into(),
    };
    let command_fn = parse_macro_input!(item as ItemFn);
    let vis = &command_fn.vis;
    let asyncness = &command_fn.sig.asyncness;
    let command_name = &command_fn.sig.ident;
    let command_name_str = command_name.to_string();
    let wrapper_name = Ident::new(&format!("bitbar_{command_name}_wrapper"), Span::call_site());
    let awaitness = asyncness.as_ref().map(|_| quote!(.await));
    let (wrapper_body, command_params, command_args) = if varargs {
        (
            quote!(::bitbar::CommandOutput::report(#command_name(args)#awaitness, #command_name_str)),
            quote!(::std::iter::Iterator::collect(::std::iter::Iterator::chain(::std::iter::once(::std::string::ToString::to_string(#command_name_str)), args))),
            quote!(_: ::bitbar::flavor::SwiftBar, args: ::std::vec::Vec<::std::string::String>),
        )
    } else {
        let mut wrapper_params = Vec::default();
        let mut wrapped_args = Vec::default();
        let mut command_params = Vec::default();
        let mut command_args = Vec::default();
        for (arg_idx, arg) in command_fn.sig.inputs.iter().enumerate() {
            match arg {
                FnArg::Receiver(_) => return quote_spanned! {arg.span()=>
                    compile_error("unexpected `self` parameter in bitbar::command");
                }.into(),
                FnArg::Typed(PatType { ty, .. }) => {
                    let ident = Ident::new(&format!("arg{}", arg_idx), arg.span());
                    wrapper_params.push(quote_spanned! {arg.span()=>
                        #ident
                    });
                    wrapped_args.push(quote_spanned! {arg.span()=>
                        match #ident.parse() {
                            ::core::result::Result::Ok(arg) => arg,
                            ::core::result::Result::Err(e) => {
                                ::bitbar::notify(e);
                                ::std::process::exit(1)
                            }
                        }
                    });
                    command_params.push(quote_spanned! {arg.span()=>
                        #ident.to_string()
                    });
                    command_args.push(quote_spanned! {arg.span()=>
                        #ident: &#ty
                    });
                }
            }
        }
        if command_args.len() > 5 {
            command_args.insert(0, quote!(_: ::bitbar::flavor::SwiftBar));
        }
        (
            quote! {
                match &*args {
                    [#(#wrapper_params),*] => ::bitbar::CommandOutput::report(#command_name(#(#wrapped_args),*)#awaitness, #command_name_str),
                    _ => {
                        ::bitbar::notify("wrong number of command arguments");
                        ::std::process::exit(1)
                    }
                }
            },
            quote!(::std::vec![
                ::std::string::ToString::to_string(#command_name_str),
                #(#command_params,)*
            ]),
            quote!(#(#command_args),*),
        )
    };
    #[cfg(not(feature = "tokio"))] let (wrapper_ret, wrapper_body) = (quote!(), wrapper_body);
    #[cfg(feature = "tokio")] let (wrapper_ret, wrapper_body) = (
        quote!(-> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()>>>),
        quote!(::std::boxed::Box::pin(async move { #wrapper_body })),
    );
    TokenStream::from(quote! {
        fn #wrapper_name(args: ::std::vec::Vec<::std::string::String>) #wrapper_ret {
            #command_fn

            #wrapper_body
        }

        #vis fn #command_name(#command_args) -> ::std::io::Result<::bitbar::attr::Params> {
            ::std::io::Result::Ok(
                ::bitbar::attr::Params::new(::std::env::current_exe()?.into_os_string().into_string().expect("non-UTF-8 plugin path"), #command_params)
            )
        }
    })
}

/// Defines a function that is called when no other `bitbar::command` matches.
///
/// * It must take as arguments the subcommand name as a `String` and the remaining arguments as a `Vec<String>`.
/// * It must return a member of the `bitbar::CommandOutput` trait.
/// * It can be a `fn` or an `async fn`. In the latter case, `tokio`'s threaded runtime will be used. (This requires the `tokio` feature, which is on by default.)
///
/// If this attribute isn't used, `bitbar` will handle unknown subcommands by sending a notification and exiting.
///
/// The function must also be registered via `#[bitbar::main(fallback_command = "...")]`.
#[proc_macro_attribute]
pub fn fallback_command(_: TokenStream, item: TokenStream) -> TokenStream {
    let fallback_fn = parse_macro_input!(item as ItemFn);
    let asyncness = &fallback_fn.sig.asyncness;
    let fn_name = &fallback_fn.sig.ident;
    let wrapper_name = Ident::new(&format!("bitbar_{fn_name}_wrapper"), Span::call_site());
    let awaitness = asyncness.as_ref().map(|_| quote!(.await));
    let wrapper_body = quote! {
        ::bitbar::CommandOutput::report(#fn_name(cmd.clone(), args)#awaitness, &cmd);
    };
    #[cfg(not(feature = "tokio"))] let (wrapper_ret, wrapper_body) = (quote!(), wrapper_body);
    #[cfg(feature = "tokio")] let (wrapper_ret, wrapper_body) = (
        quote!(-> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()>>>),
        quote!(::std::boxed::Box::pin(async move { #wrapper_body })),
    );
    TokenStream::from(quote! {
        fn #wrapper_name(cmd: ::std::string::String, args: ::std::vec::Vec<::std::string::String>) #wrapper_ret {
            #fallback_fn

            #wrapper_body
        }
    })
}

/// Annotate your `main` function with this.
///
/// * It can optionally take an argument of type `bitbar::Flavor`.
/// * It must return a member of the `bitbar::MainOutput` trait.
/// * It can be a `fn` or an `async fn`. In the latter case, `tokio`'s threaded runtime will be used. (This requires the `tokio` feature, which is on by default.)
///
/// The `main` attribute optionally takes the following parameter:
///
/// * `commands` can be set to a list of subcommand names (in parentheses) which will be used if the binary is called with command-line parameters.
/// * `fallback_command` can be set to a function name (in quotes) which will be used if the binary is called with command-line parameters and the first parameter does not match any subcommand.
/// * `error_template_image` can be set to a path (relative to the current file) to a PNG file which will be used as the template image for the menu when displaying an error.
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, Token![,]>::parse_terminated);
    let mut error_template_image = quote!(::core::option::Option::None);
    let mut fallback_lit = None;
    let mut subcommand_names = Vec::default();
    let mut subcommand_fns = Vec::default();
    for arg in args {
        if arg.path().is_ident("commands") {
            match arg.require_list() {
                Ok(list) => match list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                    Ok(nested) => for cmd in nested {
                        match cmd.require_path_only() {
                            Ok(path) => if let Some(ident) = path.get_ident() {
                                subcommand_names.push(ident.to_string());
                                subcommand_fns.push(Ident::new(&format!("bitbar_{ident}_wrapper"), ident.span()));
                            } else {
                                return quote_spanned! {cmd.span()=>
                                    compile_error!("bitbar subcommands must be simple identifiers");
                                }.into()
                            },
                            Err(e) => return e.into_compile_error().into(),
                        }
                    },
                    Err(e) => return e.into_compile_error().into(),
                }
                Err(e) => return e.into_compile_error().into(),
            }
        } else if arg.path().is_ident("error_template_image") {
            match arg.require_name_value() {
                Ok(MetaNameValue { value, .. }) => if let Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) = value {
                    error_template_image = quote!(::core::option::Option::Some(::bitbar::attr::Image::from(&include_bytes!(#lit)[..])));
                } else {
                    return quote_spanned! {value.span()=>
                        compile_error!("error_template_image value must be a string literal");
                    }.into()
                },
                Err(e) => return e.into_compile_error().into(),
            }
        } else if arg.path().is_ident("fallback_command") {
            match arg.require_name_value() {
                Ok(MetaNameValue { value, .. }) => if let Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) = value {
                    fallback_lit = Some(Ident::new(&format!("bitbar_{}_wrapper", lit.value()), lit.span()));
                } else {
                    return quote_spanned! {value.span()=>
                        compile_error!("fallback_command value must be a string literal");
                    }.into()
                },
                Err(e) => return e.into_compile_error().into(),
            }
        } else {
            return quote_spanned! {arg.span()=>
                compile_error!("unexpected bitbar::main attribute argument");
            }.into()
        }
    }
    let main_fn = parse_macro_input!(item as ItemFn);
    let asyncness = &main_fn.sig.asyncness;
    let inner_params = &main_fn.sig.inputs;
    let inner_args = if inner_params.len() >= 1 {
        quote!(::bitbar::Flavor::check())
    } else {
        quote!()
    };
    #[cfg(not(feature = "tokio"))] let (cmd_awaitness, wrapper_body) = (
        quote!(),
        quote!(::bitbar::MainOutput::main_output(main_inner(#inner_args), #error_template_image);),
    );
    #[cfg(feature = "tokio")] let awaitness = asyncness.as_ref().map(|_| quote!(.await));
    #[cfg(feature = "tokio")] let (cmd_awaitness, wrapper_body) = (
        quote!(.await),
        quote!(::bitbar::AsyncMainOutput::main_output(main_inner(#inner_args)#awaitness, #error_template_image).await;),
    );
    let fallback = if let Some(fallback_lit) = fallback_lit {
        quote!(#fallback_lit(subcommand, args.collect())#cmd_awaitness)
    } else {
        quote! {{
            ::bitbar::notify(format!("no such subcommand: {}", subcommand));
            ::std::process::exit(1)
        }}
    };
    let wrapper_body = quote!({
        //TODO set up a more friendly panic hook (similar to human-panic but rendering the panic message as a menu)
        let mut args = ::std::env::args();
        let _ = args.next().expect("missing program name");
        if let ::core::option::Option::Some(subcommand) = args.next() {
            match &*subcommand {
                #(
                    #subcommand_names => #subcommand_fns(args.collect())#cmd_awaitness,
                )*
                _ => #fallback,
            }
        } else {
            #wrapper_body
        }
    });
    #[cfg(feature = "tokio")] let wrapper_body = quote!({
        ::bitbar::tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async #wrapper_body)
    });
    let ret = main_fn.sig.output;
    let inner_body = main_fn.block;
    TokenStream::from(quote! {
        #asyncness fn main_inner(#inner_params) #ret #inner_body

        fn main() #wrapper_body
    })
}
