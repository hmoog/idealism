use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Clone0)]
pub fn delegate_clone(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let inner_type = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => {
                panic!("DelegateClone can only be derived for newtype structs with a single field")
            }
        },
        _ => panic!("DelegateClone can only be derived for structs"),
    };

    let mut generics = input.generics.clone();
    generics
        .where_clause
        .get_or_insert_with(|| syn::WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        })
        .predicates
        .push(syn::parse_quote!(#inner_type: Clone));

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Clone for #name #ty_generics #where_clause {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(Default0)]
pub fn delegate_default(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let inner_type = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => panic!(
                "DelegateDefault can only be derived for newtype structs with a single field"
            ),
        },
        _ => panic!("DelegateDefault can only be derived for structs"),
    };

    let mut generics = input.generics.clone();
    generics
        .where_clause
        .get_or_insert_with(|| syn::WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        })
        .predicates
        .push(syn::parse_quote!(#inner_type: Default));

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Default for #name #ty_generics #where_clause {
            fn default() -> Self {
                Self(Default::default())
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(Deref0)]
pub fn delegate_deref(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let inner_type = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => {
                panic!("DelegateDeref can only be derived for newtype structs with a single field")
            }
        },
        _ => panic!("DelegateDeref can only be derived for structs"),
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics std::ops::Deref for #name #ty_generics #where_clause {
            type Target = #inner_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl #impl_generics std::ops::DerefMut for #name #ty_generics #where_clause {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(FromIterator0)]
pub fn delegate_from_iterator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let inner_type = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => panic!(
                "DelegateFromIterator can only be derived for newtype structs with a single field"
            ),
        },
        _ => panic!("DelegateFromIterator can only be derived for structs"),
    };

    let (_, ty_generics, _) = input.generics.split_for_impl();

    let item_generic = syn::Ident::new("__Item", proc_macro2::Span::call_site());

    let mut generics = input.generics.clone();
    generics
        .params
        .push(syn::GenericParam::Type(syn::TypeParam {
            attrs: Vec::new(),
            ident: item_generic.clone(),
            colon_token: None,
            bounds: Default::default(),
            eq_token: None,
            default: None,
        }));

    generics
        .where_clause
        .get_or_insert_with(|| syn::WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        })
        .predicates
        .push(syn::parse_quote!(#inner_type: FromIterator<#item_generic>));

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics FromIterator<#item_generic> for #name #ty_generics #where_clause {
            fn from_iter<T: IntoIterator<Item = #item_generic>>(iter: T) -> Self {
                Self(<#inner_type as FromIterator<#item_generic>>::from_iter(iter))
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(IntoIterator0)]
pub fn delegate_into_iterator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let inner_type = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => panic!("IntoIterator can only be derived for newtype structs with a single field"),
        },
        _ => panic!("IntoIterator can only be derived for structs"),
    };

    let mut generics = input.generics.clone();
    generics
        .where_clause
        .get_or_insert_with(|| syn::WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        })
        .predicates
        .push(syn::parse_quote!(#inner_type: IntoIterator));

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut ref_generics = input.generics.clone();
    ref_generics
        .where_clause
        .get_or_insert_with(|| syn::WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        })
        .predicates
        .push(syn::parse_quote!(&'a #inner_type: IntoIterator));
    ref_generics.params.insert(0, syn::parse_quote!('a)); // Add a lifetime parameter for the reference implementations

    let (ref_impl_generics, _, ref_where_clause) = ref_generics.split_for_impl();

    let mut mut_ref_generics = input.generics.clone();
    mut_ref_generics
        .where_clause
        .get_or_insert_with(|| syn::WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        })
        .predicates
        .push(syn::parse_quote!(&'a mut #inner_type: IntoIterator));
    mut_ref_generics.params.insert(0, syn::parse_quote!('a)); // Add a lifetime parameter for the reference implementations

    let (mut_ref_impl_generics, _, mut_ref_where_clause) = mut_ref_generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics IntoIterator for #name #ty_generics #where_clause {
            type Item = <#inner_type as IntoIterator>::Item;
            type IntoIter = <#inner_type as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl #ref_impl_generics IntoIterator for &'a #name #ty_generics #ref_where_clause {
            type Item = <&'a #inner_type as IntoIterator>::Item;
            type IntoIter = <&'a #inner_type as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                (&self.0).into_iter()
            }
        }

        impl #mut_ref_impl_generics IntoIterator for &'a mut #name #ty_generics #mut_ref_where_clause {
            type Item = <&'a mut #inner_type as IntoIterator>::Item;
            type IntoIter = <&'a mut #inner_type as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                (&mut self.0).into_iter()
            }
        }
    };

    expanded.into()
}
