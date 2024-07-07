use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_str, punctuated::Punctuated, token::Comma, GenericArgument,
    GenericParam, Ident, ItemStruct, Meta, Type,
};

#[proc_macro_derive(Holdable)]
pub fn holder_derive(input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &item_struct.ident;
    let holder_trait_name = format_ident!("{}Holder", struct_name);
    let fn_name = struct_name.to_string().clone().to_case(Case::Snake);
    let mut_fn_name = format!("{}_mut", fn_name);
    let fn_name: Ident = parse_str(fn_name.as_str()).unwrap();
    let mut_fn_name: Ident = parse_str(mut_fn_name.as_str()).unwrap();
    let struct_generic = item_struct.generics.params;
    let mut struct_generic_without_bounds = struct_generic.clone();
    remove_bounds_from_generic(&mut struct_generic_without_bounds);
    let struct_where_clause = item_struct.generics.where_clause;
    let struct_visibility = item_struct.vis;
    quote! {
        #struct_visibility trait #holder_trait_name<#struct_generic> #struct_where_clause {
            fn #fn_name(&self) -> &#struct_name<#struct_generic_without_bounds>;
            fn #mut_fn_name(&mut self) -> &mut #struct_name<#struct_generic_without_bounds>;
        }
    }
    .into()
}

#[proc_macro_derive(Holder, attributes(hold))]
pub fn holder(input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &item_struct.ident;
    let struct_generic = &item_struct.generics.params;
    let mut struct_generic_without_bounds = struct_generic.clone();
    remove_bounds_from_generic(&mut struct_generic_without_bounds);
    let struct_where_clause = &item_struct.generics.where_clause;

    let quotes: Vec<_> = item_struct
        .fields
        .iter()
        .filter_map(|field| {
            let is_holdable_field = field.attrs.iter().any(|attr| match &attr.meta {
                Meta::List(list) => list.path.is_ident("hold"),
                Meta::Path(path) => path.is_ident("hold"),
                _ => panic!("unimplemented attr meta type"),
            });
            if !is_holdable_field {
                return Option::<proc_macro2::TokenStream>::None;
            }
            let field_name = field
                .ident
                .clone()
                .expect("unimplemented non field name case");
            let field_type_ident = get_ident_by_type(&field.ty);
            let field_type = &field.ty;
            let fn_name = field_type_ident.to_string().clone().to_case(Case::Snake);
            let fn_name: Ident = parse_str(fn_name.as_str()).unwrap();
            let mut_fn_name = format!("{}_mut", fn_name.to_string());
            let mut_fn_name: Ident = parse_str(mut_fn_name.as_str()).unwrap();
            let holder_trait_name = format!("{}Holder", field_type_ident);
            let holder_trait_name: Ident = parse_str(holder_trait_name.as_str()).unwrap();
            let field_bounds = get_generic_by_type(field_type);
            Some(
                quote! {
                    impl<#struct_generic>
                        #holder_trait_name<#field_bounds> for #struct_name<#struct_generic_without_bounds> #struct_where_clause {
                        fn #fn_name(&self) -> &#field_type {
                            &self.#field_name
                        }
                        fn #mut_fn_name(&mut self) -> &mut #field_type {
                            &mut self.#field_name
                        }
                    }
                }
                .into(),
            )
        })
        .collect();
    quote! {#(#quotes)*}.into()
}

fn get_ident_by_type(ty: &Type) -> Ident {
    match ty {
        Type::Path(value) => value.path.segments.last().unwrap().ident.clone(),
        Type::Reference(value) => get_ident_by_type(&*value.elem),
        _ => panic!("unimplemented field type"),
    }
}

fn get_generic_by_type(ty: &Type) -> Option<Punctuated<GenericArgument, Comma>> {
    match ty {
        Type::Path(value) => match &value.path.segments.last().unwrap().arguments {
            syn::PathArguments::None => None,
            syn::PathArguments::AngleBracketed(value) => Some(value.args.clone()),
            syn::PathArguments::Parenthesized(_) => None,
        },
        Type::Reference(value) => get_generic_by_type(&*value.elem),
        _ => panic!("unimplemented field type"),
    }
}

fn remove_bounds_from_generic(generic: &mut Punctuated<GenericParam, Comma>) {
    for struct_generic in generic.iter_mut() {
        match struct_generic {
            syn::GenericParam::Lifetime(lifetime) => {
                lifetime.bounds.clear();
            }
            syn::GenericParam::Type(lifetime) => {
                lifetime.bounds.clear();
            }
            _ => {}
        }
    }
}
