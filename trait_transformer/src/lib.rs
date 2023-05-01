// Copyright (c) 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Comma,
    Ident, ItemTrait, Path, PredicateType, Result, ReturnType, Token, TraitBound,
    TraitBoundModifier, TraitItem, Type, WherePredicate,
};

struct Attrs {
    traits: Punctuated<Transform, Comma>,
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            traits: input.parse_terminated(Transform::parse, Token![,])?,
        })
    }
}

struct Transform {
    subtrait_name: Ident,
    #[allow(dead_code)]
    colon: Token![:],
    subtrait: Path,
}

impl Parse for Transform {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            subtrait_name: input.parse()?,
            colon: input.parse()?,
            subtrait: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn trait_transformer(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attrs = parse_macro_input!(attr as Attrs);
    let item = parse_macro_input!(item as ItemTrait);

    let transformed_trait = transform_trait(&attrs, &item);
    let output = quote! {
        #item
        #transformed_trait
    };

    output.into()
}

fn transform_trait(attrs: &Attrs, tr: &ItemTrait) -> TokenStream {
    let traits = attrs
        .traits
        .iter()
        .map(|attr| {
            let mut new_trait = ItemTrait {
                attrs: Vec::new(),
                ident: attr.subtrait_name.clone(),
                items: Vec::new(),
                ..tr.clone()
            };
            new_trait
                .supertraits
                .push(syn::TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: attr.subtrait.clone(),
                }));
            let where_clause = new_trait.generics.make_where_clause();

            let subtrait = &attr.subtrait;
            for item in tr.items.iter() {
                match item {
                    TraitItem::Fn(item_fn) => {
                        let is_async = item_fn.sig.asyncness.is_some();
                        let returns_impl_trait =
                            if let ReturnType::Type(_, ty) = &item_fn.sig.output {
                                matches!(**ty, Type::ImplTrait(_))
                            } else {
                                false
                            };

                        if is_async || returns_impl_trait {
                            let name = &item_fn.sig.ident;
                            where_clause
                                .predicates
                                .push(WherePredicate::Type(PredicateType {
                                    lifetimes: None,
                                    bounded_ty: Type::Verbatim(quote!(Self::#name())),
                                    colon_token: Token![:](Span::call_site()),
                                    bounds: parse_quote!(#subtrait),
                                }));
                        }
                    }
                    _ => (),
                }
            }
            new_trait
        })
        .collect::<Vec<_>>();

    quote! { #(#traits)* }
}
