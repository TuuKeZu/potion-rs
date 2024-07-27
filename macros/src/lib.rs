use proc_macro::{TokenStream};
use routing::construct_routing_system;
use syn::{parse_macro_input, DataStruct, DeriveInput};
use syn::Data::Struct;


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

    ts.parse().unwrap()
}

#[proc_macro_derive(IntoContext)]
pub fn hello_derive(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = ast.ident;

    match ast.data {
        Struct(DataStruct { .. }) => {},
        _ => unimplemented!("Only works for structs"),
    };

    quote::quote!{
        impl potion::Context for #name {
            fn as_any(&self) -> &dyn Any {
                self
            }
        
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        
            fn box_clone(&self) -> Box<dyn potion::Context + Send + Sync> {
                Box::new((*self).clone())
            }
        }
    }.into()
}
/* 
pub fn impl_hello_world(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        impl potion::Context for #name {
            fn hello_world() {
                println!("Hello, World! My name is {}", stringify!(#name));
            }
        }
    }
}
*/
#[test]
fn test_router() {
    let a = construct_routing_system("D:\\potion-test\\src\\routing");
    dbg!(a);
}
