use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// From a struct T,  derive a struct PartialT which has every member
/// wrapped in an Option.
/// Also derive Default (=all None) and ToConcrete for mbf-deser::deserialize
#[proc_macro_attribute]
pub fn make_partial(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let original = input.clone();

    let name = &input.ident;
    let partial_name = syn::Ident::new(&format!("Partial{}", name), name.span());
    let vis = &input.vis;
    let attrs = &input.attrs;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (fields, to_concrete_impl, default_impl) = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
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

                    let field_names: Vec<_> = fields
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();

                    // Generate the to_concrete implementation
                    let to_concrete = quote! {
                        impl #impl_generics ToConcrete<#name #ty_generics> for #partial_name #ty_generics #where_clause {
                            fn to_concrete(self) -> Option<#name #ty_generics> {
                                Some(#name {
                                    #(#field_names: self.#field_names?),*
                                })
                            }
                        }
                    };

                    // Generate the Default implementation
                    let default_impl = quote! {
                        impl #impl_generics Default for #partial_name #ty_generics #where_clause {
                            fn default() -> Self {
                                Self {
                                    #(#field_names: None),*
                                }
                            }
                        }
                    };

                    (quote! { { #(#field_defs),* } }, to_concrete, default_impl)
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

                    let field_indices: Vec<_> =
                        (0..fields.unnamed.len()).map(syn::Index::from).collect();

                    // Create a vector of None tokens for default
                    let none_values = vec![quote! { None }; fields.unnamed.len()];

                    let to_concrete = quote! {
                        impl #impl_generics ToConcrete<#name #ty_generics> for #partial_name #ty_generics #where_clause {
                            fn to_concrete(self) -> Option<#name #ty_generics> {
                                Some(#name(
                                    #(self.#field_indices?),*
                                ))
                            }
                        }
                    };

                    let default_impl = quote! {
                        impl #impl_generics Default for #partial_name #ty_generics #where_clause {
                            fn default() -> Self {
                                Self(#(#none_values),*)
                            }
                        }
                    };

                    (quote! { ( #(#field_defs),* ); }, to_concrete, default_impl)
                }

                Fields::Unit => {
                    let to_concrete = quote! {
                        impl #impl_generics ToConcrete<#name #ty_generics> for #partial_name #ty_generics #where_clause {
                            fn to_concrete(self) -> Option<#name #ty_generics> {
                                Some(#name)
                            }
                        }
                    };

                    let default_impl = quote! {
                        impl #impl_generics Default for #partial_name #ty_generics #where_clause {
                            fn default() -> Self {
                                Self
                            }
                        }
                    };

                    (quote! { ; }, to_concrete, default_impl)
                }
            }
        }
        _ => panic!("make_partial only works on structs"),
    };

    let expanded = quote! {
        #original

        #(#attrs)*
        #vis struct #partial_name #generics #fields

        #to_concrete_impl
        #default_impl
    };

    TokenStream::from(expanded)
}
