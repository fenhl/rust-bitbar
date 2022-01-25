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
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::{
        quote,
        quote_spanned,
    },
    syn::{
        AttributeArgs,
        FnArg,
        Ident,
        ItemFn,
        Lit,
        Meta,
        MetaNameValue,
        NestedMeta,
        PatType,
        parse_macro_input,
        spanned::Spanned as _,
    },
};

/// Registers a subcommand that you can run from a menu item's `command`.
///
/// Commands may take any number of parameters implementing `FromStr` (with errors implementing `Display`) and `ToString`, and should return `Result<(), Error>`, where `Error` is any type that implements `Display`. If a command errors, `bitbar` will attempt to send a macOS notification containing the error message.
///
/// Alternatively, use this arrtibute as `#[command(varargs)] and define the command function with a single parameter of type `Vec<String>`.
///
/// The `command` attribute generates a function that can be called with arguments of references to the original parameter types to obtain a `std::io::Result<Params>`. If the command has more than 5 parameters or is declared with `#[command(varargs)]`, the function takes an additional first parameter of type `SwiftBar`.
///
/// Using this requires a `main` function annotated with `bitbar::main`.
#[proc_macro_attribute]
pub fn command(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let varargs = match &args[..] {
        [] => false,
        [NestedMeta::Meta(Meta::Path(path))] if path.is_ident("varargs") => true,
        _ => return quote!(compile_error!("unexpected bitbar::command arguments");).into(),
    };
    let command_fn = parse_macro_input!(item as ItemFn);
    let vis = &command_fn.vis;
    let asyncness = &command_fn.sig.asyncness;
    let command_name = &command_fn.sig.ident;
    let command_name_str = command_name.to_string();
    let wrapper_name = Ident::new(&format!("{}_wrapper", command_name), Span::call_site());
    let awaitness = asyncness.as_ref().map(|_| quote!(.await));
    let (wrapper_body, command_params, command_args) = if varargs {
        (
            quote!(::bitbar::CommandOutput::report(#command_name(args)#awaitness, #command_name_str)),
            quote!(args),
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
            quote!(vec![#(#command_params),*]),
            quote!(#(#command_args),*),
        )
    };
    #[cfg(not(any(feature = "tokio", feature = "tokio02")))] let (wrapper_ret, wrapper_body) = (quote!(), wrapper_body);
    #[cfg(any(feature = "tokio", feature = "tokio02"))] let (wrapper_ret, wrapper_body) = (
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

        inventory::submit! {
            Subcommand {
                name: #command_name_str,
                func: #wrapper_name,
            }
        }
    })
}

/// Registers a function that is called when no other `bitbar::command` matches.
///
/// * It must take as arguments the subcommand name as a `String` and the remaining arguments as a `Vec<String>`.
/// * It must return a member of the `bitbar::CommandOutput` trait.
/// * It can be a `fn` or an `async fn`. In the latter case, `tokio`'s threaded runtime will be used. (This requires the `tokio` feature, which is on by default, or either of the `tokio02` or `tokio03` features, which are not.)
///
/// If this attribute isn't used, `bitbar` will handle unknown subcommands by sending a notification and exiting.
///
/// Using this requires a `main` function annotated with `bitbar::main`.
#[proc_macro_attribute]
pub fn fallback_command(_: TokenStream, item: TokenStream) -> TokenStream {
    let fallback_fn = parse_macro_input!(item as ItemFn);
    let asyncness = &fallback_fn.sig.asyncness;
    let fn_name = &fallback_fn.sig.ident;
    let wrapper_name = Ident::new(&format!("{}_wrapper", fn_name), Span::call_site());
    let awaitness = asyncness.as_ref().map(|_| quote!(.await));
    let wrapper_body = quote! {
        ::bitbar::CommandOutput::report(#fn_name(cmd.clone(), args)#awaitness, &cmd);
    };
    #[cfg(not(any(feature = "tokio", feature = "tokio02")))] let (wrapper_ret, wrapper_body) = (quote!(), wrapper_body);
    #[cfg(any(feature = "tokio", feature = "tokio02"))] let (wrapper_ret, wrapper_body) = (
        quote!(-> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()>>>),
        quote!(::std::boxed::Box::pin(async move { #wrapper_body })),
    );
    TokenStream::from(quote! {
        fn #wrapper_name(cmd: ::std::string::String, args: ::std::vec::Vec<::std::string::String>) #wrapper_ret {
            #fallback_fn

            #wrapper_body
        }

        inventory::submit! { Fallback(#wrapper_name) }
    })
}

/// Annotate your `main` function with this.
///
/// * It can optionally take an argument of type `bitbar::Flavor`.
/// * It must return a member of the `bitbar::MainOutput` trait.
/// * It can be a `fn` or an `async fn`. In the latter case, `tokio`'s threaded runtime will be used. (This requires the `tokio` feature, which is on by default, or either of the `tokio02` or `tokio03` features, which are not.)
///
/// The `main` attribute optionally takes a parameter `error_template_image` which can be set to a path (relative to the current file) to a PNG file which will be used as the template image for the menu when displaying an error.
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let error_template_image = match &args[..] {
        [] => quote!(::core::option::Option::None),
        [NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit: Lit::Str(path_lit), ..}))]
        if path.is_ident("error_template_image") => {
            quote!(::core::option::Option::Some(::bitbar::attr::Image::from(&include_bytes!(#path_lit)[..])))
        }
        _ => return quote!(compile_error!("unexpected bitbar::main arguments");).into(),
    };
    let main_fn = parse_macro_input!(item as ItemFn);
    let asyncness = &main_fn.sig.asyncness;
    let awaitness = asyncness.as_ref().map(|_| quote!(.await));
    let inner_params = &main_fn.sig.inputs;
    let inner_args = if inner_params.len() >= 1 {
        quote!(::bitbar::Flavor::check())
    } else {
        quote!()
    };
    #[cfg(not(any(feature = "tokio", feature = "tokio02")))] let (cmd_ret, cmd_awaitness) = (quote!(), quote!());
    #[cfg(any(feature = "tokio", feature = "tokio02"))] let (cmd_ret, cmd_awaitness) = (
        quote!(-> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()>>>),
        quote!(.await),
    );
    let wrapper_body = quote!({
        //TODO set up a more friendly panic hook (similar to human-panic but rendering the panic message as a menu)
        let mut args = ::std::env::args();
        let _ = args.next().expect("missing program name");
        if let ::core::option::Option::Some(subcommand) = args.next() {
            if let ::core::option::Option::Some(command) = inventory::iter::<Subcommand>.into_iter().find(|command| command.name == &subcommand) {
                (command.func)(args.collect())#cmd_awaitness;
            } else if let ::core::option::Option::Some(Fallback(fallback)) = inventory::iter::<Fallback>.into_iter().next() {
                fallback(subcommand, args.collect())#cmd_awaitness;
            } else {
                ::bitbar::notify(format!("no such subcommand: {}", subcommand));
                ::std::process::exit(1)
            }
        } else {
            print!("{}", ::bitbar::MainOutput::main_output(main_inner(#inner_args)#awaitness, #error_template_image));
        }
    });
    #[cfg(feature = "tokio02")] let wrapper_body = quote!({
        ::bitbar::tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async #wrapper_body)
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
        use ::bitbar::inventory;

        struct Subcommand {
            name: &'static str,
            func: fn(::std::vec::Vec<::std::string::String>) #cmd_ret,
        }

        inventory::collect!(Subcommand);

        struct Fallback(fn(::std::string::String, ::std::vec::Vec<::std::string::String>) #cmd_ret);

        inventory::collect!(Fallback);

        #asyncness fn main_inner(#inner_params) #ret #inner_body

        fn main() #wrapper_body
    })
}
