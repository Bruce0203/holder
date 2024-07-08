use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_str, punctuated::Punctuated, token::Comma, Attribute, GenericArgument,
    GenericParam, Ident, Item, ItemStruct, Meta, Type, Visibility, WhereClause,
};

const HOLDER_SUFFIX: &'static str = "Holder";

struct ItemEnumOrStruct {
    ident: Ident,
    generic_params: Punctuated<GenericParam, Comma>,
    where_clause: Option<WhereClause>,
    vis: Visibility,
}
#[proc_macro_derive(Holdable)]
pub fn holder_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    let item = match item {
        Item::Enum(value) => ItemEnumOrStruct {
            ident: value.ident,
            generic_params: value.generics.params,
            where_clause: value.generics.where_clause,
            vis: value.vis,
        },
        Item::Struct(value) => ItemEnumOrStruct {
            ident: value.ident,
            generic_params: value.generics.params,
            where_clause: value.generics.where_clause,
            vis: value.vis,
        },
        _ => panic!("unimplemented item type"),
    };
    let struct_name = &item.ident;
    let struct_generic = item.generic_params;
    let struct_where_clause = item.where_clause.map(|v| v.predicates);
    let struct_visibility = item.vis;
    let mut struct_generic_without_bounds = struct_generic.clone();
    remove_bounds_from_generic(&mut struct_generic_without_bounds);
    let holder_trait_name = format_ident!("{}{HOLDER_SUFFIX}", struct_name);
    let fn_name = struct_name.to_string().clone().to_case(Case::Snake);
    let mut_fn_name = format!("{}_mut", fn_name);
    let fn_name: Ident = parse_str(fn_name.as_str()).unwrap();
    let mut_fn_name: Ident = parse_str(mut_fn_name.as_str()).unwrap();
    #[cfg(feature = "fast_delegate")]
    let attr: Attribute = parse_quote!(#[fast_delegate::delegate]);
    #[cfg(not(feature = "fast_delegate"))]
    let attr: Option<Attribute> = None;
    quote! {
        #attr
        #struct_visibility trait #holder_trait_name<#struct_generic> where #struct_where_clause {
            fn #fn_name(&self) -> &#struct_name<#struct_generic_without_bounds>;
            fn #mut_fn_name(&mut self) -> &mut #struct_name<#struct_generic_without_bounds>;
        }

        impl<__Holder, #struct_generic> #holder_trait_name<#struct_generic_without_bounds> for __Holder
            where __Holder: holder::Holder<#struct_name<#struct_generic_without_bounds>>, #struct_where_clause {

            fn #fn_name(&self) -> &#struct_name<#struct_generic_without_bounds> {
                self.get()
            }
            fn #mut_fn_name(&mut self) -> &mut #struct_name<#struct_generic_without_bounds> {
                self.get_mut()
            }
        }
    }
    .into()
}

#[proc_macro_derive(Holder, attributes(hold, hold_generic))]
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
            let mut is_generic_holder: bool = false;
            let is_holdable_field = field.attrs.iter().any(|attr| {
                let path = match &attr.meta {
                    Meta::List(list) => list.path.clone(),
                    Meta::Path(path) => path.clone(),
                    _ => panic!("unimplemented attr meta type"),
                };
                if path.is_ident("hold") {
                    true
                } else if path.is_ident("hold_generic") {
                    is_generic_holder = true;
                    true
                } else {
                    false
                }
            });
            if !is_holdable_field {
                return Option::<proc_macro2::TokenStream>::None;
            }
            let field_name = field
                .ident
                .clone()
                .expect("unimplemented non field name case");
            let field_type_ident = get_ident_by_type(&field.ty);

            let holder_trait_name: Ident = parse_str(format!("{}{HOLDER_SUFFIX}", field_type_ident).as_str()).unwrap();

            let mut type_name = holder_trait_name.clone().to_string();
            type_name.truncate(holder_trait_name.to_string().len() - HOLDER_SUFFIX.len());

            let field_type = &field.ty;
            let field_type_name: Ident = parse_str(type_name.as_str()).unwrap();
            let field_bounds = get_generic_by_type(field_type);

            let fn_name = type_name.to_case(Case::Snake);
            let fn_name: Ident = parse_str(fn_name.as_str()).unwrap();
            let mut_fn_name = format!("{}_mut", fn_name.to_string());
            let mut_fn_name: Ident = parse_str(mut_fn_name.as_str()).unwrap();

            Some(if is_generic_holder {
                quote! {
                    impl<#struct_generic> 
                        holder::Holder<#field_type_name<#field_bounds>> for #struct_name<#struct_generic_without_bounds> #struct_where_clause {
                        fn get(&self) -> &#field_type_name<#field_bounds> {
                            &self.#field_name
                        }

                        fn get_mut(&mut self) -> &mut #field_type_name<#field_bounds> {
                            &mut self.#field_name
                        }
                    }
                }
                .into()
            } else {
                quote! {
                    impl<#struct_generic>
                        #holder_trait_name<#field_bounds> for #struct_name<#struct_generic_without_bounds> #struct_where_clause {
                        fn #fn_name(&self) -> &#field_type_name<#field_bounds> {
                            &self.#field_name
                        }
                        fn #mut_fn_name(&mut self) -> &mut #field_type_name<#field_bounds> {
                            &mut self.#field_name
                        }
                    }
                }
                .into()
            })
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
