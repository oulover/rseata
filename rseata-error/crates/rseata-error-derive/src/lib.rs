extern crate proc_macro;

use darling::ast::Style;
use darling::{FromDeriveInput, FromVariant};
use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(BsServiceError, attributes(bs_service_error))]
pub fn derive_error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let receiver = InputReceiver::from_derive_input(&input).unwrap();
  TokenStream::from(quote!(#receiver))
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(bs_service_error), supports(enum_any))]
struct InputReceiver {
  ident: syn::Ident,
  generics: syn::Generics,
  data: darling::ast::Data<VariantReceiver, ()>,
  #[darling(default)]
  name: String,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(bs_service_error))]
struct VariantReceiver {
  ident: syn::Ident,
  fields: darling::ast::Fields<darling::util::SpannedValue<syn::Field>>,
  #[darling(default)]
  public: bool,
  #[darling(default)]
  public_message: Option<String>,
  #[darling(default)]
  kind: Option<syn::Ident>,
  #[darling(default)]
  name: Option<String>,
  #[darling(default)]
  #[allow(unused)]
  data_expr: Option<syn::LitStr>,
}

impl quote::ToTokens for InputReceiver {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let InputReceiver {
      ref name,
      ref ident,
      ref generics,
      ref data,
    } = *self;

    let (imp, ty, wher) = generics.split_for_impl();
    let variants = data.as_ref().take_enum().expect("Should never be struct");

    let arms = variants
      .into_iter()
      .map(|v| {
        let v_ident = &v.ident;
        let kind = v
          .kind
          .as_ref()
          .map(|name| {
            quote! {
              bs_error::BsErrorKind::#name
            }
          })
          .unwrap_or_else(|| {
            quote! {
               bs_error::BsErrorKind::Internal
            }
          });

        let name = v.name.as_ref().map(|v_name| v_name).unwrap_or_else(|| name);
        let is_public = v.public || v.public_message.is_some();
        let public_message = if is_public {
          if v.public_message.is_some() {
            let message = &v.public_message;
            quote! {
              Some(#message.into())
            }
          } else {
            quote! {
              Some(from.to_string())
            }
          }
        } else {
          quote! {
            None
          }
        };

        let data_expr_idents: Vec<_> = match v.fields.style {
          Style::Unit => vec![],
          Style::Struct => v.fields.iter().filter_map(|f| f.ident.clone()).collect(),
          Style::Tuple => v
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| syn::Ident::new(&format!("_{}", i), f.span()))
            .collect(),
        };

        let data_expr_ty: Vec<_> = match v.fields.style {
          Style::Unit => vec![],
          Style::Struct => v.fields.iter().map(|f| f.ty.clone()).collect(),
          Style::Tuple => v.fields.iter().map(|f| f.ty.clone()).collect(),
        };

        let inner_err_is_bs_error = is_bs_error(&data_expr_ty);
        if data_expr_idents.len() > 1 && inner_err_is_bs_error {
          emit_error!(data_expr_idents[0].span(), "must have only one BsError`");
        }

        let cause = quote! { from.into() };

        match v.fields.style {
          Style::Unit => {
            quote! {
              #ident::#v_ident => bs_error::BsError {
                public_message: #public_message,
                kind: #kind,
                name: #name.into(),
                cause: #cause,
              },
            }
          }
          Style::Struct => {
            quote! {
              #ident::#v_ident{#(ref #data_expr_idents),*} => bs_error::BsError {
                public_message: #public_message,
                kind: #kind,
                name: #name.into(),
                cause: #cause,
              },
            }
          }
          Style::Tuple if is_bs_error(&data_expr_ty) => {
            quote! {
              #ident::#v_ident(#(#data_expr_idents),*) => bs_error::BsError {
                public_message: _0.public_message,
                kind: _0.kind,
                name: #name.into(),
                cause: _0.cause.context(#name),
              },
            }
          }
          Style::Tuple => {
            quote! {
              #ident::#v_ident(#(ref #data_expr_idents),*) => bs_error::BsError {
                public_message: #public_message,
                kind: #kind,
                name: #name.into(),
                cause: #cause,
              },
            }
          }
        }
      })
      .collect::<Vec<_>>();

    let to_bs_error = quote! {
      impl #imp From<#ident #ty #wher> for bs_error::BsError {
        fn from(from: #ident) -> bs_error::BsError {
          match from {
            #(#arms)*
          }
        }
      }
    };

    tokens.extend(quote! {
      #to_bs_error
    });
  }
}

fn is_bs_error(types: &[syn::Type]) -> bool {
  for ty in types {
    if let syn::Type::Path(p) = ty {
      for s in p.path.segments.iter() {
        if s.ident == "BsError" {
          return true;
        }
      }
    }
  }

  false
}
