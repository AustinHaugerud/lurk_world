extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute,
    parse_macro_input,
    DeriveInput
};
use syn::Meta;
use syn::MetaNameValue;
use syn::Lit;

fn select_attribute<'a>(attrs: &'a Vec<Attribute>, name: &str) -> &'a Attribute {
    use quote::ToTokens;
    for attr in attrs.iter() {
        let meta = attr.parse_meta().expect("Failed to parse meta.");

        if let Meta::NameValue(nv) = meta {
            let attr_name = nv.path.to_token_stream().to_string();
            if &attr_name == name {
                return attr;
            }
        }
        else {
            panic!("Attribute must be name-value pair.");
        }
    }
    panic!("Failed to find '{}' attribute.", name);
}

#[proc_macro_derive(TypeCode, attributes(Code))]
pub fn type_code(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_type_code(&input)
}

fn impl_type_code(ast: &DeriveInput) -> TokenStream {
    let code_attr = select_attribute(&ast.attrs, "Code");
    let content = code_attr.tokens.to_string();

    let code: u8 = content
        .replace("=", "")
        .trim()
        .parse().expect("Code must be a valid u8 value.");

    let name = &ast.ident;
    let expanded = quote! {
        impl TypeCode for #name {
            fn type_code() -> u8 {
                #code
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(LurkReadable, attributes(StaticBlockSize, VarBlock))]
pub fn lurk_readable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let static_block_size_attr = select_attribute(&input.attrs, "StaticBlockSize");
    let var_block_attr = select_attribute(&input.attrs, "VarBlock");

    let static_block_size: usize = static_block_size_attr.tokens.to_string()
        .replace("=", "")
        .trim()
        .parse()
        .expect("StaticBlockSize must be a valid usize value.");

    let has_var_block: bool = var_block_attr.tokens.to_string()
        .replace("=", "")
        .trim()
        .parse()
        .expect("VarBlock must be a valid bool.");

    let name = &input.ident;
    let expanded = quote! {
        impl LurkReadable for #name {
            fn static_block_size() -> usize {
                #static_block_size
            }

            fn has_var_block() -> bool {
                #has_var_block
            }
        }
    };

    TokenStream::from(expanded)
}
