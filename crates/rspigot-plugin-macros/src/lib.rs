mod events;
mod listeners;
mod plugin;

use proc_macro::TokenStream;
use syn::{DeriveInput, Error, ItemImpl, parse_macro_input};

#[proc_macro_derive(Plugin, attributes(plugin))]
pub fn derive_plugin(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    plugin::derive(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn listeners(args: TokenStream, input: TokenStream) -> TokenStream {
    let block = parse_macro_input!(input as ItemImpl);
    listeners::expand(args.into(), block).into()
}
