use bromine_ecs::{Component, Resource};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, parse_macro_input};

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    let input  = parse_macro_input!(input as Item);
    todo!()
}
