//! REFLD-R003 test 1: the expansion names items only through absolute paths starting with
//! `::liminal_reflect`, `::core` or `::std` (rules (a) to (d) of the requirement).

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Expr, Token};

use super::{ACCEPTED, expand, tokens};

const ROOTS: [&str; 3] = ["liminal_reflect", "core", "std"];
const LISTED: [&str; 12] = [
    "Reflect", "Typed", "TypeInfo", "Shape", "Option", "Some", "None", "Result", "Ok", "Err",
    "Vec", "String",
];

/// Collects every violation of rules (a) to (d).
struct Checker {
    params: Vec<String>,
    violations: Vec<String>,
}

fn first(path: &syn::Path) -> String {
    path.segments
        .first()
        .map(|segment| segment.ident.to_string())
        .unwrap_or_default()
}

fn absolute_with_root(path: &syn::Path) -> bool {
    path.leading_colon.is_some() && ROOTS.contains(&first(path).as_str())
}

impl Checker {
    fn path_allowed(&self, path: &syn::Path) -> bool {
        if path.leading_colon.is_some() {
            return absolute_with_root(path);
        }
        let head = first(path);
        head == "Self"
            || self.params.contains(&head)
            || (path.segments.len() == 1 && !LISTED.contains(&head.as_str()))
    }
}

impl<'ast> Visit<'ast> for Checker {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.violations
            .push(format!("(a) use item `{}`", item.to_token_stream()));
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        self.violations
            .push(format!("(a) extern crate `{}`", item.to_token_stream()));
    }

    fn visit_qself(&mut self, qself: &'ast syn::QSelf) {
        if qself.as_token.is_none() {
            self.violations.push(format!(
                "(b) qualified self without `as`: `{}`",
                qself.ty.to_token_stream()
            ));
        }
        visit::visit_qself(self, qself);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if !absolute_with_root(&mac.path) {
            self.violations
                .push(format!("(c) macro path `{}`", mac.path.to_token_stream()));
        }
        for segment in &mac.path.segments {
            self.visit_path_arguments(&segment.arguments);
        }
        match mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated) {
            Ok(expressions) => {
                for expression in &expressions {
                    self.visit_expr(expression);
                }
            }
            Err(_) => self
                .violations
                .push(format!("(c) macro body `{}`", mac.tokens)),
        }
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if !self.path_allowed(path) {
            self.violations
                .push(format!("(d) path `{}`", path.to_token_stream()));
        }
        visit::visit_path(self, path);
    }
}

/// The violations of `output`; `params` are the input's generic type parameters.
fn violations(output: TokenStream, params: &[&str]) -> Vec<String> {
    let file: syn::File = syn::parse2(output).expect("the expansion parses as a file");
    let mut checker = Checker {
        params: params.iter().map(ToString::to_string).collect(),
        violations: Vec::new(),
    };
    checker.visit_file(&file);
    checker.violations
}

/// Verifies: REFLD-R003
#[test]
fn expansion_uses_absolute_paths_only() {
    let params: [&[&str]; 6] = [&[], &[], &[], &[], &["T", "U"], &["T"]];
    for (source, params) in ACCEPTED.iter().zip(params) {
        let output = expand(source);
        assert!(
            !output.to_string().contains("compile_error"),
            "{source} is accepted"
        );
        assert_eq!(violations(output, params), Vec::<String>::new(), "{source}");
    }
}

/// Verifies: REFLD-R003
#[test]
fn path_check_is_not_vacuous() {
    let core =
        tokens("const _: () = { fn f() -> u64 { ::liminal_core::hash::state_hash_of(&1_u8) } };");
    let found = violations(core, &[]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("(d) path `:: liminal_core"),
        "{found:?}"
    );

    let option = tokens("const _: () = { fn f() -> Option<u8> { ::core::option::Option::None } };");
    let found = violations(option, &[]);
    assert_eq!(found, ["(d) path `Option < u8 >`"]);

    let others = tokens(
        "use ::core::fmt as _; const _: () = { fn f<T>() { <T>::g(); format!(\"\"); ::std::format!(\"{}\", Vec::<u8>::new()); } };",
    );
    let found = violations(others, &["T"]);
    let rules: Vec<_> = found.iter().map(|v| &v[..3]).collect();
    assert_eq!(rules, ["(a)", "(b)", "(d)", "(c)", "(d)"], "{found:?}");
}
