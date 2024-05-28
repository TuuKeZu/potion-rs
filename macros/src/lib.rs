use proc_macro::{Span, TokenStream};
use routing::construct_routing_system;

mod file_system;
mod routing;

#[proc_macro]
pub fn routing(_p: TokenStream) -> TokenStream {
    let potion_routing_dir = dotenv::dotenv_iter()
        .map(|env| {
            env.into_iter().flatten().find_map(|(k, v)| {
                if k == "POTION_ROUTING_DIR" {
                    Some(v)
                } else {
                    None
                }
            })
        })
        .unwrap_or(None);

    if potion_routing_dir.is_none() {
        panic!("'POTION_ROUTING_DIR' is unset. Are you sure you have \".env\" file correctly configured?")
    }

    let ts = construct_routing_system(&potion_routing_dir.unwrap())
        .expect("Failed to construct module tree");
    dbg!(&ts);
    ts.parse().unwrap()
}
