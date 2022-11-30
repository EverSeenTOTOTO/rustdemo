use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_quote, visit_mut::VisitMut, Expr, ItemFn, Lit};

// visit and modify AST
struct Visitor;

impl VisitMut for Visitor {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        match &node {
            Expr::Lit(lit_expr) => {
                if let Lit::Int(lit) = &lit_expr.lit {
                    let value = lit.base10_parse::<u16>().unwrap();

                    match value {
                        0 => {
                            *node = parse_quote! {1};
                        }
                        1 => {
                            *node = parse_quote! {1};
                        }
                        _ => {
                            let minus_1 = value - 1;
                            let minus_2 = value - 2;

                            // println!("{}, {}", minus_1, minus_2);

                            *node = parse_quote! { (fib!(#minus_1) + fib!(#minus_2)) }
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }
    }

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
    let mut item = syn::parse(input).expect("failed to parse input");

    Visitor.visit_item_fn_mut(&mut item);

    let output = quote! { #item };
    return output.into();
}

#[proc_macro]
pub fn fib(input: TokenStream) -> TokenStream {
    let mut item = syn::parse(input).expect("failed to parse input");

    Visitor.visit_expr_mut(&mut item);

    let output = quote! { #item };
    return output.into();
}
