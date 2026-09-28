use proc_macro::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, parse_macro_input};

/// Runs the use case's `execute` method in a transaction.
///
/// Place this attribute before `#[UseCase]`. SQLx gateways use the active transaction
/// connection; the use case only needs conversions from `GatewayError`
/// and `UnexpectedError` into its error type.
#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn Transactional(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new(
            proc_macro::Span::call_site().into(),
            "Transactional takes no arguments",
        )
        .into_compile_error()
        .into();
    }

    let mut implementation = parse_macro_input!(input as ItemImpl);
    let Some(execute) = implementation.items.iter_mut().find_map(|item| match item {
        ImplItem::Fn(method) if method.sig.ident == "execute" => Some(method),
        _ => None,
    }) else {
        return syn::Error::new_spanned(
            &implementation,
            "Transactional requires an execute method",
        )
        .into_compile_error()
        .into();
    };

    if execute.sig.asyncness.is_none() || execute.sig.receiver().is_none() {
        return syn::Error::new_spanned(
            &execute.sig,
            "Transactional requires an async execute method with a receiver",
        )
        .into_compile_error()
        .into();
    }

    let body = &execute.block;
    execute.block = match syn::parse2(quote!({
        crate::infrastructure::transaction::run_in_transaction(async move #body).await
    })) {
        Ok(block) => block,
        Err(error) => return error.into_compile_error().into(),
    };

    quote!(#implementation).into()
}
