use proc_macro::{TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Rebindable)]
pub fn derive_rebindable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;

    let mut impl_params: Vec<Box<dyn ToTokens>> = vec![Box::new(quote! { 'a })];
    let mut type_params: Vec<Box<dyn ToTokens>> = vec![];
    let mut out_params: Vec<Box<dyn ToTokens>> = vec![];

    for _ in generics.lifetimes() {
        type_params.push(Box::new(quote! { '_ }));
        out_params.push(Box::new(quote! { 'a }));
    }

    for ident in generics.type_params().map(|p| &p.ident) {
        impl_params.push(Box::new(quote! { #ident: 'a }));
        type_params.push(Box::new(ident.clone()));
        out_params.push(Box::new(ident.clone()));
    }

    for param in generics.const_params() {
        let ident = &param.ident;
        let ty = &param.ty;
        impl_params.push(Box::new(quote! { const #ident: #ty }));
        type_params.push(Box::new(ident.clone()));
        out_params.push(Box::new(ident.clone()));
    }

    TokenStream::from(quote! {
        unsafe impl<#(#impl_params),*> escher::RebindTo<'a> for #name<#(#type_params),*> {
           type Out = #name<#(#out_params),*>;
        }

        impl escher::Rebindable for #name<#(#type_params),*> {
            fn rebind<'short, 'long: 'short>(&'long self) -> &'short escher::Rebind<'short, Self>
            where Self: 'long
            {
                self
            }
        }
    })
}
