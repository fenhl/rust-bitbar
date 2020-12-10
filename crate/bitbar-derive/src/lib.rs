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
        ReturnType,
        Type,
        TypePath,
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
    let command_name = &command_fn.sig.ident;
    let command_name_str = command_name.to_string();
    let wrapper_name = Ident::new(&format!("{}_wrapper", command_name), Span::call_site());
    TokenStream::from(quote! {
        fn #wrapper_name(program: ::std::ffi::OsString, args: Vec<::std::ffi::OsString>) {
            #command_fn

            match #command_name(args.into_iter()) {
                Ok(subcommand) => {},
                Err(e) => {
                    bitbar_notify(format!("{}: {}", #command_name_str, e));
                    ::std::process::exit(1);
                }
            }
        }

        inventory::submit! {
            Subcommand {
                name: #command_name_str,
                func: #wrapper_name,
            }
        }
    })
}

/// Annotate your `main` function with this.
///
/// * It can be a `fn` or an `async fn`. In the latter case, `tokio`'s threaded runtime will be used. (This requires the `tokio` feature, which is on by default, or the `tokio02` feature, which is not.)
/// * It must return a `Menu` or a `Result<Menu, E>`, for some `E` that implements `Into<Menu>`. The `Result` in the type signature must be unqualified (`::std::result::Result` will not work).
///
/// The `main` attribute optionally takes a parameter `error_template_image` which can be set to a path (relative to the current file) to a PNG file which will be used as the template image for the menu when displaying an error.
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let error_template_image = match &args[..] {
        [] => quote!(),
        [NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit: Lit::Str(path_lit), ..}))]
        if path.get_ident().map_or(false, |arg| arg.to_string() == "error_template_image") => {
            quote!(.template_image(&include_bytes!(#path_lit)[..]).never_unwrap())
        }
        _ => return TokenStream::from(quote!(compile_error!("unexpected bitbar::main arguments"))),
    };
    let main_fn = parse_macro_input!(item as ItemFn);
    let asyncness = &main_fn.sig.asyncness;
    let use_tokio = asyncness.as_ref().map(|_| quote!(use ::bitbar::tokio;));
    let main_prefix = asyncness.as_ref().map(|async_keyword| quote!(#[tokio::main] #async_keyword));
    let awaitness = asyncness.as_ref().map(|_| quote!(.await));
    let ret = main_fn.sig.output;
    let mut main_ret_match_body = quote! {
        menu => print!("{}", menu),
    };
    if let ReturnType::Type(_, ref ty) = ret {
        if let Type::Path(TypePath { qself: None, ref path }) = **ty {
            if path.segments.len() == 1 && path.segments[0].ident.to_string() == "Result" {
                main_ret_match_body = quote! {
                    ::core::result::Result::Ok(menu) => print!("{}", menu),
                    ::core::result::Result::Err(e) => {
                        let mut menu = Menu(vec![
                            ::bitbar::ContentItem::new("?")#error_template_image.into(),
                            ::bitbar::MenuItem::Sep,
                        ]);
                        menu.extend(::bitbar::Menu::from(e));
                        print!("{}", menu);
                    }
                }
            }
        }
    };
    let body = main_fn.block;
    TokenStream::from(quote! {
        use ::bitbar::inventory;
        #use_tokio

        struct Subcommand {
            name: &'static str,
            func: fn(::std::ffi::OsString, Vec<::std::ffi::OsString>),
        }

        inventory::collect!(Subcommand);

        #asyncness fn main_inner() #ret #body

        fn bitbar_notify(body: impl ::std::fmt::Display) {
            //let _ = notify_rust::set_application(&notify_rust::get_bundle_identifier_or_default("BitBar")); //TODO uncomment when https://github.com/h4llow3En/mac-notification-sys/issues/8 is fixed
            let _ = ::bitbar::notify_rust::Notification::default()
                .summary(&env!("CARGO_PKG_NAME"))
                .sound_name("Funky")
                .body(&body.to_string())
                .show();
        }

        #main_prefix fn main() {
            //TODO set up a more friendly panic hook (similar to human-panic but rendering the panic message as a menu)
            let mut args = ::std::env::args_os();
            let program = args.next().expect("missing program name");
            if let ::core::option::Option::Some(subcommand) = args.next() {
                let subcommand = match subcommand.into_string() {
                    ::core::result::Result::Ok(subcommand) => subcommand,
                    ::core::result::Result::Err(_) => {
                        bitbar_notify("subcommand is not valid UTF-8");
                        ::std::process::exit(1);
                    }
                };
                if let ::core::option::Option::Some(command) = inventory::iter::<Subcommand>.into_iter().find(|command| command.name == subcommand) {
                    (command.func)(program, args.collect());
                } else {
                    bitbar_notify(format!("no such subcommand: {}", subcommand));
                    ::std::process::exit(1);
                }
            } else {
                match main_inner()#awaitness { #main_ret_match_body }
            }
        }
    })
}
