use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_attribute]
pub fn make_partial(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let original = input.clone();

    let name = &input.ident;
    let partial_name = syn::Ident::new(&format!("Partial{}", name), name.span());
    let vis = &input.vis;
    let attrs = &input.attrs;
    let generics = &input.generics;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let field_defs = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;
                    let vis = &f.vis;
                    let attrs = &f.attrs;
                    quote! {
                        #(#attrs)*
                        #vis #name: Option<#ty>
                    }
                });
                quote! { { #(#field_defs),* } }
            }
            Fields::Unnamed(fields) => {
                let field_defs = fields.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    let vis = &f.vis;
                    let attrs = &f.attrs;
                    quote! {
                        #(#attrs)*
                        #vis Option<#ty>
                    }
                });
                quote! { ( #(#field_defs),* ); }
            }
            Fields::Unit => quote! { ; },
        },
        _ => panic!("make_partial only works on structs"),
    };

    let expanded = quote! {
        #original

        #(#attrs)*
        #vis struct #partial_name #generics #fields
    };

    TokenStream::from(expanded)
}
