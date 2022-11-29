use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_quote, visit_mut::VisitMut, ItemFn};

// visit and modify AST
struct FnVisitor;

impl VisitMut for FnVisitor {
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
        let old_block = &node.block;
        let output = &node.sig.output;

        node.block = Box::new(parse_quote! {
            {
                if let Err(_) = (|| #output #old_block)() { // #output 是原本的返回类型，#old_block
                                                            // 是原本的函数体
                    println!("got you");
                }
                return Ok(());
            }
        })
    }
}

#[proc_macro_attribute]
pub fn nothrow(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: ItemFn = syn::parse(input).expect("failed to parse input");

    FnVisitor.visit_item_fn_mut(&mut item);

    let output = quote! { #item };
    return output.into();
}
