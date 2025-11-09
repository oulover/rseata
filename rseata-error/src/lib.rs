/// `BsServiceError` macro
/// Implements From<BsServiceError> for given enum
/// # Examples
/// ```
/// // You can have rust code between fences inside the comments
/// // If you pass --test to `rustdoc`, it will even test it for you!
/// use bs_error::{BsError, BsServiceError};
/// #[derive(BsServiceError)]
/// // Default name that can be overriden in fields
/// #[bs_service_error(name = "design")]
/// pub enum Error {
/// #[bs_service_error(public_message = "Hello", kind = "Internal", name = "override enum level name")]
/// Ha,
/// #[bs_service_error(public, kind = "NotFound", name = "foo")]
/// Foo(String),
/// // Specific scenario maps directly to BsError adding `name` as the context
/// blaa(BsError),
///}
/// ```
pub use bs_error_derive::BsServiceError;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub struct BsError {
  pub kind: BsErrorKind,
  pub name: String,
  pub public_message: Option<String>,
  pub cause: anyhow::Error,
}

impl fmt::Display for BsError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.cause)
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BsErrorKind {
  InvalidArgument,
  PermissionDenied,
  Unauthenticated,
  Internal,
  NotFound,
}

impl BsErrorKind {
  pub fn is_public(&self) -> bool {
    match self {
      Self::InvalidArgument | Self::PermissionDenied | Self::Unauthenticated | Self::NotFound => {
        true
      }
      _ => false,
    }
  }
}

pub type Result<T, E = BsError> = std::result::Result<T, E>;

impl From<tonic::Status> for BsError {
  fn from(from: tonic::Status) -> Self {
    let kind = match from.code() {
      tonic::Code::InvalidArgument => BsErrorKind::InvalidArgument,
      tonic::Code::NotFound => BsErrorKind::NotFound,
      tonic::Code::PermissionDenied => BsErrorKind::PermissionDenied,
      tonic::Code::Unauthenticated => BsErrorKind::Unauthenticated,
      tonic::Code::AlreadyExists => BsErrorKind::InvalidArgument,
      _ => BsErrorKind::Internal,
    };
    let is_public = kind.is_public();
    Self {
      kind,
      name: String::new(),
      public_message: {
        if is_public {
          Some(from.message().to_string())
        } else {
          None
        }
      },
      cause: from.into(),
    }
  }
}
