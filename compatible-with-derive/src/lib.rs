use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(CompatibleWith)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;
    let crate_name = syn::Ident::new("compatible_with", proc_macro2::Span::call_site());
    quote::quote! {
        impl<Old> From<#crate_name::Compatible<Old, #name>> for #name
        where
            Self: #crate_name::CompatibleWith<Old>,
        {
            fn from(value: #crate_name::Compatible<Old, #name>) -> Self {
                value.into_current()
            }
        }
    }
    .into()
}
