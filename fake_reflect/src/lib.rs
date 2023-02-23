extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{Data, DataStruct, DeriveInput, Fields, Ident};
use std::any::Any;

fn impl_hello_world(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let field_name = fields.iter().map(|field| &field.ident);
    let field_name2 = field_name.clone();
    let field_name3 = field_name.clone();
    let field_name4 = field_name.clone();
    let field_name5 = field_name.clone();
    let field_name6 = field_name.clone();
    let field_name7 = field_name.clone();

    let field_type = fields.iter().map(|field| &field.ty);

    let enum_name = syn::parse_str::<Ident>(&format!("{}_field", name.to_string())).unwrap();

    let enum_vars = fields.iter().map(|field| {
        syn::parse_str::<syn::Path>(&format!("{}::{}", enum_name, field.ident.as_ref().unwrap()))
            .unwrap()
    });

    TokenStream::from(quote! {
        //#ast
        pub enum #enum_name {
            #(
                #field_name(#field_type),
            )*
        }

        impl #name {
            pub fn get(&self, field: impl ToString) -> anyhow::Result<#enum_name> {
                match field.to_string().as_str() {
                    #(
                        stringify!(#field_name2) => Ok(#enum_vars(self.#field_name3.clone())),
                    )*
                    _ => Err(anyhow::anyhow!("Unknown field.")),
                }
            }

           pub fn update(self, field: impl ToString, val: &str) -> Result<Self> {
               match field.to_string().as_str() {
                    #(
                        stringify!(#field_name4) => {
                            let mut new = self.clone();
                            new.#field_name5 = val.parse().unwrap();
                            Ok(new)
                        },
                     )*
                    _ => Err(anyhow::anyhow!("Unknown field.")),
               }
           }
        }
    })
}

#[proc_macro_derive(hello_world)]
pub fn hello_world(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse_macro_input!(input as DeriveInput);

    // Build and return the generated impl
    impl_hello_world(ast)
}