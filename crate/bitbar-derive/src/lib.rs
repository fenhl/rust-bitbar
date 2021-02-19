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
    quote::quote,
    syn::{
        AttributeArgs,
        Ident,
        ItemFn,
        Lit,
        Meta,
        MetaNameValue,
        NestedMeta,
        parse_macro_input,
    },
};

/// Registers a subcommand that you can run from a menu item's `command`.
///
/// Commands should have the signature `fn(impl Iterator<Item = OsString>) -> Result<(), Error>`, where `Error` is any type that implements `Display`. If a command errors, `bitbar` will attempt to send a macOS notification containing the error message.
///
/// Using this requires a `main` function annotated with `bitbar::main`.
#[proc_macro_attribute]
pub fn command(_: TokenStream, item: TokenStream) -> TokenStream {
    let command_fn = parse_macro_input!(item as ItemFn);
    let asyncness = &command_fn.sig.asyncness;
    let command_name = &command_fn.sig.ident;
    let command_name_str = command_name.to_string();
    let wrapper_name = Ident::new(&format!("{}_wrapper", command_name), Span::call_site());
    let awaitness = asyncness.as_ref().map(|_| quote!(.await));
    let wrapper_body = quote! {
        ::bitbar::CommandOutput::report(#command_name(args.into_iter())#awaitness, #command_name_str);
    };
    #[cfg(not(any(feature = "tokio", feature = "tokio02")))] let (wrapper_ret, wrapper_body) = (quote!(), wrapper_body);
    #[cfg(any(feature = "tokio", feature = "tokio02"))] let (wrapper_ret, wrapper_body) = (
        quote!(-> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()>>>),
        quote!(::std::boxed::Box::pin(async move { #wrapper_body })),
    );
    TokenStream::from(quote! {
        fn #wrapper_name(program: ::std::ffi::OsString, args: Vec<::std::ffi::OsString>) #wrapper_ret {
            #command_fn

            #wrapper_body
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
/// * It must take as arguments the subcommand name as an `OsString` and the remaining arguments as an `impl Iterator<Item = OsString>`.
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
        let first_arg = args.first().expect("missing subcommand to report on").to_str().unwrap_or("<invalid UTF-8>").to_owned();
        ::bitbar::CommandOutput::report(#fn_name(cmd, args.into_iter())#awaitness, &first_arg);
    };
    #[cfg(not(any(feature = "tokio", feature = "tokio02")))] let (wrapper_ret, wrapper_body) = (quote!(), wrapper_body);
    #[cfg(any(feature = "tokio", feature = "tokio02"))] let (wrapper_ret, wrapper_body) = (
        quote!(-> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()>>>),
        quote!(::std::boxed::Box::pin(async move { #wrapper_body })),
    );
    TokenStream::from(quote! {
        fn #wrapper_name(cmd: ::std::ffi::OsString, args: Vec<::std::ffi::OsString>) #wrapper_ret {
            #fallback_fn

            #wrapper_body
        }

        inventory::submit! { Fallback(#wrapper_name) }
    })
}

/// Annotate your `main` function with this.
///
/// * It must take no arguments.
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
        if path.get_ident().map_or(false, |arg| arg.to_string() == "error_template_image") => {
            quote!(::core::option::Option::Some(::bitbar::Image::from(&include_bytes!(#path_lit)[..])))
        }
        _ => return TokenStream::from(quote!(compile_error!("unexpected bitbar::main arguments"))),
    };
    let main_fn = parse_macro_input!(item as ItemFn);
    let asyncness = &main_fn.sig.asyncness;
    let awaitness = asyncness.as_ref().map(|_| quote!(.await));
    #[cfg(not(any(feature = "tokio", feature = "tokio02")))] let (cmd_ret, cmd_awaitness) = (quote!(), quote!());
    #[cfg(any(feature = "tokio", feature = "tokio02"))] let (cmd_ret, cmd_awaitness) = (
        quote!(-> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()>>>),
        quote!(.await),
    );
    let wrapper_body = quote!({
        //TODO set up a more friendly panic hook (similar to human-panic but rendering the panic message as a menu)
        let mut args = ::std::env::args_os();
        let program = args.next().expect("missing program name");
        if let ::core::option::Option::Some(subcommand) = args.next() {
            let subcommand_str = subcommand.to_str().unwrap_or_else(|| {
                ::bitbar::notify("subcommand is not valid UTF-8");
                ::std::process::exit(1)
            });
            if let ::core::option::Option::Some(command) = inventory::iter::<Subcommand>.into_iter().find(|command| command.name == subcommand_str) {
                (command.func)(program, args.collect())#cmd_awaitness;
            } else if let ::core::option::Option::Some(Fallback(fallback)) = inventory::iter::<Fallback>.into_iter().next() {
                fallback(subcommand, args.collect())#cmd_awaitness;
            } else {
                ::bitbar::notify(format!("no such subcommand: {}", subcommand_str));
                ::std::process::exit(1)
            }
        } else {
            print!("{}", ::bitbar::MainOutput::main_output(main_inner()#awaitness, #error_template_image));
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
            func: fn(::std::ffi::OsString, Vec<::std::ffi::OsString>) #cmd_ret,
        }

        inventory::collect!(Subcommand);

        struct Fallback(fn(::std::ffi::OsString, Vec<::std::ffi::OsString>) #cmd_ret);

        inventory::collect!(Fallback);

        #asyncness fn main_inner() #ret #inner_body

        fn main() #wrapper_body
    })
}
