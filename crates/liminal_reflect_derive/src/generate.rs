//! Generation of the impls for an accepted input (spec §5, REFLD-R001 to REFLD-R035).
//!
//! Every path is absolute (`::liminal_reflect`, `::core`, `::std`; REFLD-R003). Field types are
//! copied as written; the calls that need `F: Typed` carry the field type's span (REFLD-R053).

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

use crate::parse::{Base, Body, Field, Fields, Kind, Model, Variant};

/// The generated block `const _: () = { … };` (REFLD-R001).
pub(crate) fn generate(model: &Model) -> TokenStream {
    let ident = &model.ident;
    let mut generics = model.generics.clone();
    let where_clause = generics.make_where_clause();
    for param in &model.type_params {
        where_clause
            .predicates
            .push(syn::parse_quote!(#param: ::liminal_reflect::value::Typed));
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let header = |trait_name: TokenStream| {
        quote! {
            #[automatically_derived]
            impl #impl_generics #trait_name for #ident #type_generics #where_clause
        }
    };
    let reflect = header(quote!(::liminal_reflect::value::Reflect));
    let typed = header(quote!(::liminal_reflect::value::Typed));
    let fields_header = header(quote!(::liminal_reflect::value::ReflectFields));
    let view = match model.body {
        Body::Struct(_) => quote!(Struct),
        Body::Enum(_) => quote!(Enum),
    };
    let type_path = type_path(model);
    let shape = shape(&model.body);
    let (tables, fields_body, enum_impl) = match &model.body {
        Body::Struct(fields) => (
            struct_table(fields),
            struct_fields(fields),
            TokenStream::new(),
        ),
        Body::Enum(variants) => {
            let enum_header = header(quote!(::liminal_reflect::value::ReflectEnum));
            let enum_body = enum_methods(variants);
            (
                enum_table(variants),
                enum_fields(variants),
                quote!(#enum_header { #enum_body }),
            )
        }
    };
    quote! {
        const _: () = {
            #tables

            #reflect {
                fn reflect_type_info(&self) -> ::liminal_reflect::info::TypeInfo {
                    <Self as ::liminal_reflect::value::Typed>::type_info()
                }
                fn reflect_ref(&self) -> ::liminal_reflect::value::ReflectRef<'_> {
                    ::liminal_reflect::value::ReflectRef::#view(self)
                }
                fn reflect_mut(&mut self) -> ::liminal_reflect::value::ReflectMut<'_> {
                    ::liminal_reflect::value::ReflectMut::#view(self)
                }
            }

            #typed {
                fn type_path() -> ::std::string::String {
                    #type_path
                }
                fn type_info() -> ::liminal_reflect::info::TypeInfo {
                    ::liminal_reflect::info::TypeInfo {
                        type_path: <Self as ::liminal_reflect::value::Typed>::type_path(),
                        id: <Self as ::liminal_reflect::value::Typed>::stable_type_id(),
                        shape: #shape,
                    }
                }
            }

            #fields_header {
                #fields_body
            }

            #enum_impl
        };
    }
}

/// The body of `Typed::type_path` (REFLD-R020, REFLD-R021).
fn type_path(model: &Model) -> TokenStream {
    let params = &model.type_params;
    if params.is_empty() {
        return match &model.base {
            Base::Module(name) => {
                let suffix = format!("::{name}");
                quote!(::std::string::String::from(
                    ::core::concat!(::core::module_path!(), #suffix)
                ))
            }
            Base::Override(path) => quote!(::std::string::String::from(#path)),
        };
    }
    let placeholders = vec!["{}"; params.len()].join(", ");
    let arguments = params
        .iter()
        .map(|param| quote!(<#param as ::liminal_reflect::value::Typed>::type_path()));
    match &model.base {
        Base::Module(name) => {
            let format = format!("{{}}::{name}<{placeholders}>");
            quote!(::std::format!(#format, ::core::module_path!(), #(#arguments),*))
        }
        Base::Override(path) => {
            let format = format!("{path}<{placeholders}>");
            quote!(::std::format!(#format, #(#arguments),*))
        }
    }
}

fn kind(kind: Kind) -> TokenStream {
    match kind {
        Kind::Named => quote!(::liminal_reflect::info::StructKind::Named),
        Kind::Tuple => quote!(::liminal_reflect::info::StructKind::Tuple),
        Kind::Unit => quote!(::liminal_reflect::info::StructKind::Unit),
    }
}

/// `::std::vec![FieldInfo { … }, …]` of a field list (REFLD-R030).
fn field_infos(fields: &Fields) -> TokenStream {
    let infos = fields.fields.iter().map(|field| {
        let name = &field.name;
        let ty = &field.ty;
        let type_ref = quote_spanned! {ty.span()=>
            ::liminal_reflect::info::TypeRef::of::<#ty>()
        };
        quote!(::liminal_reflect::info::FieldInfo { name: #name, ty: #type_ref })
    });
    quote!(::std::vec![#(#infos),*])
}

/// The `Shape` expression of the descriptor (REFLD-R030).
fn shape(body: &Body) -> TokenStream {
    match body {
        Body::Struct(fields) => {
            let kind = kind(fields.kind);
            let infos = field_infos(fields);
            quote!(::liminal_reflect::info::Shape::Struct { kind: #kind, fields: #infos })
        }
        Body::Enum(variants) => {
            let variants = variants.iter().map(|variant| {
                let name = &variant.name;
                let kind = kind(variant.fields.kind);
                let infos = field_infos(&variant.fields);
                quote!(::liminal_reflect::info::VariantInfo { name: #name, kind: #kind, fields: #infos })
            });
            quote!(::liminal_reflect::info::Shape::Enum {
                variants: ::std::vec![#(#variants),*]
            })
        }
    }
}

fn names(fields: &Fields) -> TokenStream {
    let names = fields.fields.iter().map(|field| &field.name);
    quote!(&[#(#names),*])
}

/// The field-name table of a struct.
fn struct_table(fields: &Fields) -> TokenStream {
    let names = names(fields);
    quote! {
        const FIELD_NAMES: &[&::core::primitive::str] = #names;
    }
}

/// The field-name table (per variant) and variant-name table of an enum.
fn enum_table(variants: &[Variant]) -> TokenStream {
    let per_variant = variants.iter().map(|variant| names(&variant.fields));
    let variant_names = variants.iter().map(|variant| &variant.name);
    quote! {
        const FIELD_NAMES: &[&[&::core::primitive::str]] = &[#(#per_variant),*];
        const VARIANT_NAMES: &[&::core::primitive::str] = &[#(#variant_names),*];
    }
}

/// `ReflectFields` methods of a struct (REFLD-R032).
fn struct_fields(fields: &Fields) -> TokenStream {
    let indices = 0..fields.fields.len();
    let members: Vec<_> = fields.fields.iter().map(|field| &field.member).collect();
    let indices_mut = indices.clone();
    let accessors = if fields.fields.is_empty() {
        quote! {
            fn field_at(&self, _index: ::core::primitive::usize)
                -> ::core::option::Option<&dyn ::liminal_reflect::value::Reflect> {
                ::core::option::Option::None
            }
            fn field_at_mut(&mut self, _index: ::core::primitive::usize)
                -> ::core::option::Option<&mut dyn ::liminal_reflect::value::Reflect> {
                ::core::option::Option::None
            }
        }
    } else {
        quote! {
            fn field_at(&self, index: ::core::primitive::usize)
                -> ::core::option::Option<&dyn ::liminal_reflect::value::Reflect> {
                match index {
                    #(#indices => ::core::option::Option::Some(&self.#members),)*
                    _ => ::core::option::Option::None,
                }
            }
            fn field_at_mut(&mut self, index: ::core::primitive::usize)
                -> ::core::option::Option<&mut dyn ::liminal_reflect::value::Reflect> {
                match index {
                    #(#indices_mut => ::core::option::Option::Some(&mut self.#members),)*
                    _ => ::core::option::Option::None,
                }
            }
        }
    };
    quote! {
        fn field_count(&self) -> ::core::primitive::usize {
            FIELD_NAMES.len()
        }
        fn field_name(&self, index: ::core::primitive::usize)
            -> ::core::option::Option<&'static ::core::primitive::str> {
            FIELD_NAMES.get(index).copied()
        }
        #accessors
    }
}

/// The pattern `Self::V`, `Self::V(..)` or `Self::V { .. }` of a variant.
fn variant_pattern(variant: &Variant) -> TokenStream {
    let ident = &variant.ident;
    match variant.fields.kind {
        Kind::Unit => quote!(Self::#ident),
        Kind::Tuple => quote!(Self::#ident(..)),
        Kind::Named => quote!(Self::#ident { .. }),
    }
}

/// The arms `(Self::V { m: f, .. }, i) => Some(f)` of every reflected field of every variant.
fn field_arms(variants: &[Variant]) -> impl Iterator<Item = TokenStream> + '_ {
    let binding = format_ident!("__liminal_reflect_field");
    variants.iter().flat_map(move |variant| {
        let ident = &variant.ident;
        let binding = binding.clone();
        variant
            .fields
            .fields
            .iter()
            .enumerate()
            .map(move |(index, Field { member, .. })| {
                quote! {
                    (Self::#ident { #member: #binding, .. }, #index) =>
                        ::core::option::Option::Some(#binding),
                }
            })
    })
}

/// `ReflectFields` methods of an enum's current variant (REFLD-R032).
fn enum_fields(variants: &[Variant]) -> TokenStream {
    let current = quote! {
        FIELD_NAMES[<Self as ::liminal_reflect::value::ReflectEnum>::variant_index(self) as ::core::primitive::usize]
    };
    let has_fields = variants
        .iter()
        .any(|variant| !variant.fields.fields.is_empty());
    let accessors = if has_fields {
        let arms: Vec<_> = field_arms(variants).collect();
        quote! {
            fn field_at(&self, index: ::core::primitive::usize)
                -> ::core::option::Option<&dyn ::liminal_reflect::value::Reflect> {
                match (self, index) {
                    #(#arms)*
                    _ => ::core::option::Option::None,
                }
            }
            fn field_at_mut(&mut self, index: ::core::primitive::usize)
                -> ::core::option::Option<&mut dyn ::liminal_reflect::value::Reflect> {
                match (self, index) {
                    #(#arms)*
                    _ => ::core::option::Option::None,
                }
            }
        }
    } else {
        quote! {
            fn field_at(&self, _index: ::core::primitive::usize)
                -> ::core::option::Option<&dyn ::liminal_reflect::value::Reflect> {
                ::core::option::Option::None
            }
            fn field_at_mut(&mut self, _index: ::core::primitive::usize)
                -> ::core::option::Option<&mut dyn ::liminal_reflect::value::Reflect> {
                ::core::option::Option::None
            }
        }
    };
    quote! {
        fn field_count(&self) -> ::core::primitive::usize {
            #current.len()
        }
        fn field_name(&self, index: ::core::primitive::usize)
            -> ::core::option::Option<&'static ::core::primitive::str> {
            #current.get(index).copied()
        }
        #accessors
    }
}

/// `ReflectEnum` methods (REFLD-R031): declaration positions, not discriminants.
fn enum_methods(variants: &[Variant]) -> TokenStream {
    let patterns = variants.iter().map(variant_pattern);
    let indices = (0..variants.len()).map(|index| u32::try_from(index).unwrap_or(u32::MAX));
    quote! {
        fn variant_index(&self) -> ::core::primitive::u32 {
            match self {
                #(#patterns => #indices,)*
            }
        }
        fn variant_name(&self) -> &'static ::core::primitive::str {
            VARIANT_NAMES[<Self as ::liminal_reflect::value::ReflectEnum>::variant_index(self) as ::core::primitive::usize]
        }
    }
}
