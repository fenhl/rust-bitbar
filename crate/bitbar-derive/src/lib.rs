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
        Ident,
        ItemFn,
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
/// * It can be a `fn` or an `async fn`. In the latter case, `tokio`'s threaded runtime will be used.
/// * It must return a `Result<Menu, E>`, for some `E` that implements `Into<Menu>`.
#[proc_macro_attribute]
pub fn main(_: TokenStream, item: TokenStream) -> TokenStream { //TODO parse template image for error menu from attribute param
    let main_fn = parse_macro_input!(item as ItemFn);
    let asyncness = &main_fn.sig.asyncness;
    let main_prefix = if let Some(async_keyword) = asyncness {
        quote!(#[tokio::main] #async_keyword)
    } else {
        quote!()
    };
    let awaitness = asyncness.as_ref().map(|_| quote!(.await));
    let ret = main_fn.sig.output;
    let body = main_fn.block;
    TokenStream::from(quote! {
        use ::bitbar::{
            inventory,
            tokio,
        };

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
            let mut args = env::args_os();
            let program = args.next().expect("missing program name");
            if let Some(subcommand) = args.next() {
                let subcommand = match subcommand.into_string() {
                    Ok(subcommand) => subcommand,
                    Err(_) => {
                        bitbar_notify("subcommand is not valid UTF-8");
                        ::std::process::exit(1);
                    }
                };
                if let Some(command) = inventory::iter::<Subcommand>.into_iter().find(|command| command.name == subcommand) {
                    (command.func)(program, args.collect());
                } else {
                    bitbar_notify(format!("no such subcommand: {}", subcommand));
                    ::std::process::exit(1);
                }
            } else {
                match main_inner()#awaitness {
                    Ok(menu) => print!("{}", menu),
                    Err(e) => {
                        let mut menu = Menu(vec![
                            ::bitbar::MenuItem::new("?"), //TODO add template image
                            ::bitbar::MenuItem::Sep,
                        ]);
                        menu.extend(::bitbar::Menu::from(e));
                        print!("{}", menu);
                    }
                }
            }
        }
    })
}
