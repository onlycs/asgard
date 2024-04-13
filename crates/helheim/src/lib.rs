extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
#[macro_use]
extern crate skuld;
extern crate syn;

use proc_macro::TokenStream as StdTokenStream;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Data, DeriveInput, Error, Ident,
    LitStr, Meta, Result, Token,
};

type WArgs = Punctuated<LitStr, Token![,]>;

fn helheim(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        ident,
        data: Data::Enum(data),
        ..
    } = input
    else {
        bail!(Error::new(input.span(), "Expected enum"));
    };

    let variants = &data.variants;
    let mut arms = vec![];

    for variant in variants {
        let attrs = &variant.attrs;
        let wattr = attrs.iter().find(|attr| attr.path().is_ident("warning"));

        let Some(wattr) = wattr else {
            bail!(Error::new(
                variant.span(),
                "Expected #[warning(...)] attribute"
            ))
        };

        let Meta::List(args) = &wattr.meta else {
            bail!(Error::new(
                wattr.span(),
                "Expected a list (i.e. #[warning(...)]) attribute"
            ))
        };

        let Ok(args) = args.parse_args_with(WArgs::parse_terminated) else {
            bail!(Error::new(args.span(), "Failed to parse these arguments"));
        };

        let Some(display) = args.first() else {
            bail!(Error::new(
                args.span(),
                "Expected only one argument in this attribute, found none"
            ))
        };

        if args.len() != 1 {
            bail!(Error::new(
                args.span(),
                "Expected only one argument in this attribute"
            ))
        }

        let format_lit = display
            .value()
            .split("{")
            .enumerate()
            .map(|(n, fmt)| {
                if n == 0 {
                    fmt.to_string()
                } else {
                    format!("{{f{fmt}")
                }
            })
            .fold(String::new(), |acc, fmt| acc + &fmt);

        let format_lit = LitStr::new(&format_lit, Span::call_site());

        let fields = variant.fields.iter().enumerate().map(|(i, field)| {
            if let Some(ident) = &field.ident {
                let prefixed = Ident::new(&format!("f{}", i), Span::call_site());
                quote! { #ident: #prefixed }
            } else {
                let prefixed = Ident::new(&format!("f{}", i), Span::call_site());
                quote! { #prefixed }
            }
        });

        let arm = match variant.fields.iter().next() {
            Some(field) if field.ident.is_some() => quote! { { #(#fields),* } },
            Some(_) => quote! { ( #(#fields),* ) },
            None => quote! {},
        };

        arms.push(quote! {
            #ident::#variant #arm => format!(#format_lit),
        });
    }

    Ok(quote! {
        impl ::std::fmt::Display for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let s = match self {
                    #(#arms)*
                };
                write!(f, "{}", s)
            }
        }

        impl #ident {
            pub fn emit(&self) {
                ::log::warn!("{}", self);
            }

            pub fn into_emit(self) {
                self.emit();
            }
        }
    })
}

/// # Helheim
///
/// Basically just a Display implementation, but for warnings. Works like `thiserror`. Wraps the `log` crate
///
/// ## Example
/// ```
/// use helheim::Warning;
///
/// #[derive(Warning)]
/// enum MyWarning {
///    #[warning("Something went wrong")]
///    Something,
/// }
///
/// let warning = MyWarning::Something;
///
/// warning.emit(); // log::warn!("Something went wrong");
/// ```
#[proc_macro_derive(Warning, attributes(warning))]
pub fn warning(input: StdTokenStream) -> StdTokenStream {
    helheim(parse_macro_input!(input as DeriveInput))
        .map(StdTokenStream::from)
        .map_err(Error::into_compile_error)
        .unwrap_or_else(StdTokenStream::from)
}
