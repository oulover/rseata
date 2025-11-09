use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Error, ItemFn, LitStr};

#[proc_macro_attribute]
pub fn global_transaction(
    attr: TokenStream,
    func: TokenStream,
) -> TokenStream {
    let transaction_name = parse_macro_input!(attr as LitStr);
    let mut input_fn = parse_macro_input!(func as ItemFn);

    if input_fn.sig.asyncness.is_none() {
        return Error::new(
            input_fn.sig.span(),
            "global_transaction can only be applied to async functions",
        ).to_compile_error().into();
    }

    let _is_result = if let syn::ReturnType::Type(_, ref ty) = input_fn.sig.output {
        if let syn::Type::Path(type_path) = &**ty {
            type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident == "Result")
                .unwrap_or(false)
        } else {
            false
        }
    } else {
        false
    };

    let original_block = &input_fn.block;
    let new_block = quote! {
       {
           use rseata::core::TransactionManager;
           use rseata::FutureExt;
           use std::panic::AssertUnwindSafe;
           use rseata::RSEATA_TM;
           use std::sync::Arc;
           use rseata::RSEATA_CLIENT_SESSION;
           use rseata::core::{ClientSession};
            let session = Arc::new(ClientSession::new(String::from(#transaction_name)));
            let session_clone = session.clone();
           
            let result = RSEATA_CLIENT_SESSION.scope(
                session,
                AssertUnwindSafe(async {
                    { #original_block }
                })
                .catch_unwind()
                .map(|res| res.unwrap_or_else(|_| Err(anyhow::anyhow!("Panic occurred in transaction scope")))),
            ).await;
           
            let xid = session_clone.get_xid();
            if let Some (xid) = xid {
                match result {
                    Ok(data) => {
                        RSEATA_TM.commit(xid.clone()).await?;
                        Ok(data)
                    }
                    Err(err) => {
                        RSEATA_TM.rollback(xid.clone()).await?;
                        Err(err)
                    }
                }
            }else {
                result
            }
      }
    };
    input_fn.block = syn::parse2(new_block).unwrap();
    TokenStream::from(quote! { #input_fn })
}