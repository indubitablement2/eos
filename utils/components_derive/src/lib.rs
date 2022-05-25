extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Data, DeriveInput, Error};

#[proc_macro_derive(Components)]
pub fn components_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let data = match ast.data {
        Data::Struct(ref data) => data,
        _ => {
            let e = Error::new_spanned(
                &ast,
                "trait `Components` can only be implemented for structs",
            );
            return proc_macro::TokenStream::from(e.to_compile_error());
        }
    };

    let fields = match data.fields {
        syn::Fields::Named(ref fields) => fields,
        _ => {
            let e = Error::new_spanned(
                &ast,
                "trait `Components` can only be implemented for structs with named fields",
            );
            return proc_macro::TokenStream::from(e.to_compile_error());
        }
    };

    let move_to_table_tokens = fields.named.iter().map(|f| {
        let name = f.ident.as_ref().unwrap().to_owned();
        quote!(raw_table.ptr(Self::#name).add(index).write(self.#name);)
    });

    let move_from_table_tokens = fields.named.iter().map(|f| {
        let name = f.ident.as_ref().unwrap().to_owned();
        quote!(#name: raw_table.ptr(Self::#name).add(index).read(),)
    });

    let result = quote!(
        unsafe impl #impl_generics ::utils::Components for #ident #ty_generics #where_clause {
            unsafe fn move_to_table(self, raw_table: &mut RawTable<Self>, index: usize) {
                #(#move_to_table_tokens)*
            }

            unsafe fn move_from_table(raw_table: &mut RawTable<Self>, index: usize) -> Self {
                Self {
                    #(#move_from_table_tokens)*
                }
            }
        }
    );

    proc_macro::TokenStream::from(result)
}
