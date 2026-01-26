use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Entity, attributes(entity))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut table_name: Option<String> = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("entity") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table") {
                let lit: syn::LitStr = meta.value()?.parse()?;
                table_name = Some(lit.value());
            }
            Ok(())
        })
        .unwrap();
    }

    let Some(table_name) = table_name else {
        return syn::Error::new_spanned(&input.ident, "missing #[entity(table = \"...\")]")
            .to_compile_error()
            .into();
    };

    TokenStream::from(quote! {
        impl Entity for #name {
            fn table_name() -> &'static str {
                #table_name
            }
        }
    })
}
