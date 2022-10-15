extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, ItemStruct};

/// Create a new struct prefixed with `Soa` that implement `Container`. 
#[proc_macro_derive(Soa)]
pub fn soa_derive(tokens: TokenStream) -> TokenStream {
    let ast: ItemStruct = syn::parse(tokens).unwrap();

    let ident = &ast.ident;
    let soa_ident = format_ident!("{}Soa", ident);
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let fields = match ast.fields {
        syn::Fields::Named(ref fields) => fields,
        _ => {
            let e = Error::new_spanned(
                &ast,
                "trait `Soa` can only be implemented for structs with named fields",
            );
            panic!("{}", e);
        }
    };

    // Soa struct definition.
    let soa_fields_def = fields.named.iter().map(|f| {
        let field = f.ident.as_ref().unwrap();
        let ty = f.ty.to_owned();
        quote!(pub #field : Vec<#ty>,)
    });

    // Container impl

    let with_capacity = fields.named.iter().map(|f| {
        let field = f.ident.as_ref().unwrap();
        quote!(#field : Vec::with_capacity(capacity),)
    });

    let push = fields.named.iter().map(|f| {
        let field = f.ident.as_ref().unwrap();
        quote!(self. #field .push(value. #field);)
    });

    let pop = fields.named.iter().map(|f| {
        let field = f.ident.as_ref().unwrap();
        quote!(#field : self. #field . pop()?,)
    });

    let swap_remove = fields.named.iter().map(|f| {
        let field = f.ident.as_ref().unwrap();
        quote!(#field: self. #field .swap_remove(index) ,)
    });

    let replace = fields.named.iter().map(|f| {
        let field = f.ident.as_ref().unwrap();
        quote!(std::mem::swap(self. #field .get_mut(index).unwrap(), &mut value. #field) ;)
    });

    let len = fields
        .named
        .iter()
        .next()
        .map(|f| {
            let field = f.ident.as_ref().unwrap();
            quote!(self. #field .len())
        })
        .expect("need at least one field");

    let capacity = fields
        .named
        .iter()
        .next()
        .map(|f| {
            let field = f.ident.as_ref().unwrap();
            quote!(self. #field .capacity())
        })
        .expect("need at least one field");

    let reserve = fields.named.iter().map(|f| {
        let field = f.ident.as_ref().unwrap();
        quote!(self. #field .reserve(additional);)
    });

    let swap_elements = fields.named.iter().map(|f| {
        let field = f.ident.as_ref().unwrap();
        quote!(self. #field.swap(a, b);)
    });

    // Put it all together.
    let result = quote!(
        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        pub struct #soa_ident {
            #(#soa_fields_def)*
        }

        impl #impl_generics ::utils::packed_map::Container for #soa_ident #ty_generics #where_clause {
            type Item = #ident;

            fn with_capacity(capacity: usize) -> Self {
                Self {
                    #(#with_capacity)*
                }
            }

            fn push(&mut self, value: Self::Item) {
                #(#push)*
            }

            fn pop(&mut self) -> Option<Self::Item> {
                Some(Self::Item {
                    #(#pop)*
                })
            }

            fn swap_remove(&mut self, index: usize) -> Self::Item {
                Self::Item {
                    #(#swap_remove)*
                }
            }

            fn replace(&mut self, index: usize, mut value: Self::Item) -> Self::Item {
                #(#replace)*
                value
            }

            fn len(&self) -> usize {
                #len
            }

            fn capacity(&self) -> usize {
                #capacity
            }

            fn reserve(&mut self, additional: usize) {
                #(#reserve)*
            }

            fn swap_elements(&mut self, a: usize, b: usize) {
                #(#swap_elements)*
            }
        }
    );

    proc_macro::TokenStream::from(result)
}
