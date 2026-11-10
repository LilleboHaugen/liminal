//! Parsing and validation of the derive input (spec §4, §5, §8).
//!
//! [`parse`] turns the input tokens into a [`Model`] or returns every diagnostic of the input in
//! source order (REFLD-R051): the type's attributes, the `union` keyword or the enum identifier,
//! the generic parameters, then the fields and variants in declaration order.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Expr, Generics, Ident, Lit, LitStr, Member, Meta, Token};

/// An accepted input.
pub(crate) struct Model {
    /// The type's identifier as written (`r#` kept, for generated code).
    pub(crate) ident: Ident,
    /// The type's generics and `where` clause, unchanged.
    pub(crate) generics: Generics,
    /// The generic type parameters in declaration order.
    pub(crate) type_params: Vec<Ident>,
    /// The base path (spec §4).
    pub(crate) base: Base,
    /// Struct fields or enum variants.
    pub(crate) body: Body,
}

/// The base path of the type path (REFLD-R020, REFLD-R021).
pub(crate) enum Base {
    /// `module_path!()` + `::` + this identifier (without `r#`).
    Module(String),
    /// The value of `#[reflect(type_path = "…")]`.
    Override(String),
}

/// The data of the type.
pub(crate) enum Body {
    /// A struct and its fields.
    Struct(Fields),
    /// An enum and its variants in declaration order (at least one).
    Enum(Vec<Variant>),
}

/// The kind of a struct or variant by its syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// `{ a: T }`.
    Named,
    /// `(T)`.
    Tuple,
    /// No fields.
    Unit,
}

/// The reflected fields of a struct or variant.
pub(crate) struct Fields {
    /// The syntax of the struct or variant.
    pub(crate) kind: Kind,
    /// Reflected fields only, in declaration order (index = position in this list).
    pub(crate) fields: Vec<Field>,
}

/// One reflected field.
pub(crate) struct Field {
    /// The member used to access it (`a`, or the tuple position counted with ignored fields).
    pub(crate) member: Member,
    /// The field name of REFLD-R004.
    pub(crate) name: String,
    /// The field type as written.
    pub(crate) ty: syn::Type,
}

/// One enum variant.
pub(crate) struct Variant {
    /// The variant identifier as written.
    pub(crate) ident: Ident,
    /// The identifier without `r#`.
    pub(crate) name: String,
    /// The reflected fields of the variant.
    pub(crate) fields: Fields,
}

/// One item of a `#[reflect(…)]` list.
enum Item {
    /// `ignore`.
    Ignore,
    /// `type_path = "…"`; the key and the literal.
    TypePath(Ident, LitStr),
}

impl Item {
    fn key(&self) -> &'static str {
        match self {
            Item::Ignore => "ignore",
            Item::TypePath(..) => "type_path",
        }
    }
}

/// A diagnostic `LIM-REFLD-{code}: {message} help: {help}` spanned at `tokens`.
fn diagnostic(tokens: impl ToTokens, code: u16, message: &str, help: &str) -> syn::Error {
    syn::Error::new_spanned(tokens, text(code, message, help))
}

/// A diagnostic spanned at one `span`.
fn diagnostic_at(span: Span, code: u16, message: &str, help: &str) -> syn::Error {
    syn::Error::new(span, text(code, message, help))
}

fn text(code: u16, message: &str, help: &str) -> String {
    format!("LIM-REFLD-{code:04}: {message} help: {help}")
}

/// Parses and validates the input.
///
/// # Errors
///
/// Every diagnostic of the input, in source order (REFLD-R050, REFLD-R051).
pub(crate) fn parse(input: TokenStream) -> Result<Model, Vec<syn::Error>> {
    let input: DeriveInput = syn::parse2(input).map_err(|error| vec![error])?;
    let mut errors = Vec::new();
    let base = type_attributes(&input, &mut errors);
    match &input.data {
        Data::Union(data) => {
            errors.push(diagnostic(
                data.union_token,
                1,
                "`#[derive(Reflect)]` does not support unions.",
                "use a struct or an enum.",
            ));
        }
        Data::Enum(data) if data.variants.is_empty() => {
            let message = format!(
                "`#[derive(Reflect)]` needs at least one variant in enum `{}`.",
                input.ident.unraw()
            );
            errors.push(diagnostic(
                &input.ident,
                2,
                &message,
                "add a variant, or do not derive `Reflect` for an uninhabited type.",
            ));
        }
        _ => {}
    }
    let type_params = generics(&input.generics, &mut errors);
    let body = match &input.data {
        Data::Struct(data) => Some(Body::Struct(fields(&data.fields, &mut errors))),
        Data::Enum(data) if !data.variants.is_empty() => Some(Body::Enum(
            data.variants
                .iter()
                .map(|variant| {
                    for attribute in reflect_attributes(&variant.attrs) {
                        misplaced(attribute, "a variant", &mut errors);
                    }
                    Variant {
                        ident: variant.ident.clone(),
                        name: variant.ident.unraw().to_string(),
                        fields: fields(&variant.fields, &mut errors),
                    }
                })
                .collect(),
        )),
        _ => None,
    };
    match body {
        Some(body) if errors.is_empty() => Ok(Model {
            ident: input.ident.clone(),
            generics: input.generics.clone(),
            type_params,
            base: base.unwrap_or_else(|| Base::Module(input.ident.unraw().to_string())),
            body,
        }),
        _ => Err(errors),
    }
}

/// The `reflect` attributes of a list.
fn reflect_attributes(attributes: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("reflect"))
}

/// The items of one `reflect` attribute, or LIM-REFLD-0005.
fn items(attribute: &Attribute) -> Result<Vec<Item>, syn::Error> {
    let malformed = |tokens: String, span: Span| {
        let message = format!("unknown or malformed attribute `#[reflect({tokens})]`.");
        diagnostic_at(
            span,
            5,
            &message,
            "the accepted forms are `#[reflect(type_path = \"a::B\")]` on the type and \
             `#[reflect(ignore)]` on a field.",
        )
    };
    let list = match &attribute.meta {
        Meta::List(list) => list,
        Meta::Path(path) => return Err(malformed(String::new(), path.span())),
        Meta::NameValue(pair) => {
            let tokens = pair.eq_token.to_token_stream();
            let mut tokens = tokens.to_string();
            tokens.push(' ');
            tokens.push_str(&pair.value.to_token_stream().to_string());
            return Err(malformed(tokens, pair.eq_token.span));
        }
    };
    let span = list
        .tokens
        .clone()
        .into_iter()
        .next()
        .map_or_else(|| list.delimiter.span().open(), |token| token.span());
    let error = || malformed(list.tokens.to_string(), span);
    let metas = list
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .map_err(|_| error())?;
    if metas.is_empty() {
        return Err(error());
    }
    metas
        .iter()
        .map(|meta| match meta {
            Meta::Path(path) if path.is_ident("ignore") => Ok(Item::Ignore),
            Meta::NameValue(pair) if pair.path.is_ident("type_path") => match &pair.value {
                Expr::Lit(literal) => match &literal.lit {
                    Lit::Str(value) => Ok(Item::TypePath(
                        pair.path.get_ident().cloned().ok_or_else(error)?,
                        value.clone(),
                    )),
                    _ => Err(error()),
                },
                _ => Err(error()),
            },
            _ => Err(error()),
        })
        .collect()
}

/// Reports each item of a `reflect` attribute that is not allowed at `place` (LIM-REFLD-0008),
/// or the attribute's LIM-REFLD-0005.
fn misplaced(attribute: &Attribute, place: &str, errors: &mut Vec<syn::Error>) {
    match items(attribute) {
        Ok(items) => {
            for item in items {
                errors.push(misplaced_error(attribute, item.key(), place));
            }
        }
        Err(error) => errors.push(error),
    }
}

fn misplaced_error(attribute: &Attribute, key: &str, place: &str) -> syn::Error {
    let message = format!("`#[reflect({key})]` is not allowed on {place}.");
    diagnostic(
        attribute,
        8,
        &message,
        "put `type_path` on the type and `ignore` on fields.",
    )
}

/// The type's `reflect` attributes: the `type_path` override, if any and valid.
fn type_attributes(input: &DeriveInput, errors: &mut Vec<syn::Error>) -> Option<Base> {
    let mut base = None;
    let mut seen = false;
    for attribute in reflect_attributes(&input.attrs) {
        let items = match items(attribute) {
            Ok(items) => items,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        for item in items {
            match item {
                Item::Ignore => errors.push(misplaced_error(attribute, "ignore", "a type")),
                Item::TypePath(key, value) => {
                    if seen {
                        errors.push(diagnostic(
                            key,
                            7,
                            "`type_path` is given more than once.",
                            "keep one `#[reflect(type_path = \"…\")]`.",
                        ));
                        continue;
                    }
                    seen = true;
                    let path = value.value();
                    if is_type_path(&path) {
                        base = Some(Base::Override(path));
                    } else {
                        let message = format!(
                            "type path `{path}` is not a Rust path with at least two segments."
                        );
                        errors.push(diagnostic(
                            &value,
                            6,
                            &message,
                            "write segments of letters, digits and `_` separated by `::`, for \
                             example `my_game::Health`.",
                        ));
                    }
                }
            }
        }
    }
    base
}

/// `^[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)+$` (REFLD-R021).
pub(crate) fn is_type_path(path: &str) -> bool {
    let mut segments = 0_usize;
    for segment in path.split("::") {
        let mut chars = segment.chars();
        let head = chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
        if !head || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
        segments += 1;
    }
    segments >= 2
}

/// Checks the generic parameters; returns the type parameters.
fn generics(generics: &Generics, errors: &mut Vec<syn::Error>) -> Vec<Ident> {
    let mut type_params = Vec::new();
    for param in &generics.params {
        let attributes = match param {
            syn::GenericParam::Type(param) => &param.attrs,
            syn::GenericParam::Lifetime(param) => &param.attrs,
            syn::GenericParam::Const(param) => &param.attrs,
        };
        for attribute in reflect_attributes(attributes) {
            misplaced(attribute, "a generic parameter", errors);
        }
        match param {
            syn::GenericParam::Type(param) => type_params.push(param.ident.clone()),
            syn::GenericParam::Lifetime(param) => {
                let message = format!(
                    "`#[derive(Reflect)]` does not support lifetime parameter `{}`: reflected \
                     types are `'static`.",
                    param.lifetime
                );
                errors.push(diagnostic(
                    &param.lifetime,
                    3,
                    &message,
                    "store owned data (`String`, `Vec<T>`) instead of references.",
                ));
            }
            syn::GenericParam::Const(param) => {
                let message = format!(
                    "`#[derive(Reflect)]` does not support const parameter `{}`.",
                    param.ident
                );
                errors.push(diagnostic_at(
                    param.const_token.span,
                    4,
                    &message,
                    "use a fixed-size array field `[T; N]` with a literal `N`, or a `Vec<T>`.",
                ));
            }
        }
    }
    type_params
}

/// The reflected fields of a struct or variant; reports misplaced and repeated attributes.
fn fields(fields: &syn::Fields, errors: &mut Vec<syn::Error>) -> Fields {
    let kind = match fields {
        syn::Fields::Named(_) => Kind::Named,
        syn::Fields::Unnamed(_) => Kind::Tuple,
        syn::Fields::Unit => Kind::Unit,
    };
    let mut reflected = Vec::new();
    for (position, field) in fields.iter().enumerate() {
        let (member, name) = match &field.ident {
            Some(ident) => (Member::Named(ident.clone()), ident.unraw().to_string()),
            None => (
                Member::Unnamed(syn::Index {
                    index: u32::try_from(position).unwrap_or(u32::MAX),
                    span: Span::call_site(),
                }),
                position.to_string(),
            ),
        };
        let mut ignored = false;
        for attribute in reflect_attributes(&field.attrs) {
            let items = match items(attribute) {
                Ok(items) => items,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };
            for item in items {
                match item {
                    Item::Ignore if ignored => {
                        let message = format!(
                            "field `{name}` is marked `#[reflect(ignore)]` more than once."
                        );
                        errors.push(diagnostic(
                            attribute,
                            9,
                            &message,
                            "keep one `#[reflect(ignore)]`.",
                        ));
                    }
                    Item::Ignore => ignored = true,
                    Item::TypePath(..) => {
                        errors.push(misplaced_error(attribute, "type_path", "a field"));
                    }
                }
            }
        }
        if !ignored {
            reflected.push(Field {
                member,
                name,
                ty: field.ty.clone(),
            });
        }
    }
    Fields {
        kind,
        fields: reflected,
    }
}
