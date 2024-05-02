#![feature(proc_macro_span)]

#[cfg(feature = "routing")]
use proc_macro::{Span, TokenStream};
use routing::construct_routing_system;

mod file_system;
mod routing;

#[cfg(feature = "routing")]
#[proc_macro]
pub fn routing(_p: TokenStream) -> TokenStream {
    let ts = construct_routing_system("D:\\rannasta-suomeen-rs\\src\\routing").expect("Failed to construct module tree");

    ts.parse().unwrap()
}

#[cfg(feature = "routing")]
#[proc_macro]
pub fn do_something(item: TokenStream) -> TokenStream {
    let span = Span::call_site();
    let source = span.source_file();
    format!("println!(r#\"Path: {}\"#)", source.path().to_str().unwrap())
        .parse()
        .unwrap()
}



