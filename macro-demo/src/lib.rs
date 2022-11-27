#![feature(proc_macro_diagnostic)]

use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{Fields, Item, ItemStruct};

#[proc_macro_attribute]
pub fn attr_demo(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse(input).expect("failed to parse input");

    match item {
        Item::Struct(ref struct_item) => {
            if has_foo(struct_item) {
                raise(struct_item);
            }
        }

        _ => unreachable!(),
    }

    let output = quote! { #item };
    return output.into();
}

fn has_foo(s: &ItemStruct) -> bool {
    match s.fields {
        Fields::Named(ref fields) => fields
            .named
            .iter()
            .any(|field| return field.ident.as_ref().unwrap() == "foo"),
        _ => false,
    }
}

fn raise(s: &ItemStruct) {
    if let Fields::Named(ref fields) = s.fields {
        for field in &fields.named {
            let ident = field.ident.as_ref().unwrap();

            if ident == "foo" {
                ident.span().unstable().error("got it").emit();
            }
        }
    }
}
