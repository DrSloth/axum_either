//! Accept and respond with one of multiple types in axum handlers.
//!
//! This is espacially useful to create endpoints which take one of multiple message formats.
//! This can be done with [`AxumEither`] directly. This can get quite verbose for more than two
//! types. For this the [`one_of`] macro can be used to express the type and the [`map_one_of`] or
//! [`match_one_of`] macros can be used to work with these types ergonomically.
//!
//! # Example
//! ```
//! use axum::{Json, Form};
//! use axum_either::AxumEither;
//!
//! #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
//! pub struct Request(u32);
//!
//! /// Using AxumEither directly
//! pub async fn form_or_json(
//!     request: AxumEither<Json<Request>, Form<Request>>
//! ) -> AxumEither<Json<Request>, String> {
//!     match request {
//!         AxumEither::Left(Json(l)) => AxumEither::Left(Json(l)),
//!         AxumEither::Right(r) => AxumEither::Right(format!("{:?}", r)),
//!     }
//! }
//!
//! /// Using one_of and match_one_of
//! pub async fn request_type(
//!     request: axum_either::one_of!(Json<Request>, Form<Request>, String)
//! ) -> &'static str {
//!     axum_either::match_one_of!{request,
//!         Json(_j) => "Json",
//!         Form(_f) => "Form",
//!         _s => "String",
//!     }
//! }
//! ```
//! For more examples see the
//! [examples](https://github.com/DrSloth/axum_either/tree/master/examples) directory.

use axum_core::{
    extract::{FromRequest, RequestParts},
    response::{IntoResponse, Response},
};
use http::{header, status::StatusCode, HeaderValue};

/// Extract or Respond with one of the given types, this can be composed to extract more types.
///
/// This implements [`IntoResponse`](axum_core::response::IntoResponse) if both L and R implement
/// [`IntoResponse`]. If L and R implement [`FromRequest`] this type also does.
///
/// Requests are parsed from left to right, if both types collide the Left type is preferred.
#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum AxumEither<L, R> {
    /// The first possibility to parse, this variant is always tried first when parsing
    Left(L),
    /// The second possibility, fallback types should be `R` because this variant is checked last
    Right(R),
}

impl<L, R> AxumEither<L, R> {
    /// Maps the left value using the given function `f`.
    ///
    /// # Examples
    /// ```
    /// # use axum_either::AxumEither;
    /// let l: AxumEither<i32, bool> = AxumEither::Left(10);
    /// assert_eq!(l.map_left(|i| i * 10), AxumEither::Left(100));
    /// let r: axum_either::one_of!(i32, bool) = AxumEither::Right(false);
    /// assert_eq!(r.map_left(|i| i * 100), AxumEither::Right(false));
    /// ```
    pub fn map_left<U>(self, f: impl FnOnce(L) -> U) -> AxumEither<U, R> {
        match self {
            Self::Left(l) => AxumEither::Left(f(l)),
            Self::Right(r) => AxumEither::Right(r),
        }
    }

    /// Maps the right value using the given function `f`.
    ///
    /// # Examples
    /// ```
    /// # use axum_either::AxumEither;
    /// let l: AxumEither<i32, bool> = AxumEither::Left(10);
    /// assert_eq!(l.map_right(|b| !b), AxumEither::Left(10));
    /// let r: axum_either::one_of!(i32, bool) = AxumEither::Right(false);
    /// assert_eq!(r.map_right(|b| !b), AxumEither::Right(true));
    /// ```
    pub fn map_right<U>(self, f: impl FnOnce(R) -> U) -> AxumEither<L, U> {
        match self {
            Self::Left(l) => AxumEither::Left(l),
            Self::Right(r) => AxumEither::Right(f(r)),
        }
    }

    /// Map both the left and right values with the given `lf` and `rf` functions.
    ///
    /// # Examples
    /// ```
    /// # use axum_either::AxumEither;
    /// let l: AxumEither<i32, bool> = AxumEither::Left(10);
    /// assert_eq!(l.map_lr(|i| i * 10, |b| !b), AxumEither::Left(100));
    /// let r: axum_either::one_of!(i32, bool) = AxumEither::Right(false);
    /// assert_eq!(r.map_lr(|i| i * 10, |b| !b), AxumEither::Right(true));
    /// ```
    pub fn map_lr<L2, R2>(
        self,
        lf: impl FnOnce(L) -> L2,
        rf: impl FnOnce(R) -> R2,
    ) -> AxumEither<L2, R2> {
        self.map_left(lf).map_right(rf)
    }

    /// Extract the left value and discard the right value, Right maps to [`None`]
    ///
    /// ```
    /// # use axum_either::{AxumEither, one_of};
    /// let l: AxumEither<i32, bool> = AxumEither::Left(10);
    /// assert!(l.left().is_some());
    /// let r: axum_either::one_of!(i32, bool) = AxumEither::Right(false);
    /// assert!(r.left().is_none());
    /// ```
    pub fn left(self) -> Option<L> {
        match self {
            Self::Left(l) => Some(l),
            Self::Right(_r) => None,
        }
    }

    /// Extract the right value and discard the left value, Left maps to [`None`]
    ///
    /// # Examples
    /// ```
    /// # use axum_either::AxumEither;
    /// let l: AxumEither<i32, bool> = AxumEither::Left(10);
    /// assert!(l.right().is_none());
    /// let r: axum_either::one_of!(i32, bool) = AxumEither::Right(false);
    /// assert!(r.right().is_some());
    /// ```
    pub fn right(self) -> Option<R> {
        match self {
            Self::Left(_l) => None,
            Self::Right(r) => Some(r),
        }
    }

    /// Convert this [`AxumEither`] into a [`either::Either`]
    ///
    /// # Examples
    /// ```
    /// # use axum_either::AxumEither;
    /// let l: AxumEither<i32, bool> = AxumEither::Left(10);
    /// assert_eq!(l.into_either(), either::Either::Left(10));
    /// let r: axum_either::one_of!(i32, bool) = AxumEither::Right(false);
    /// assert_eq!(r.into_either(), either::Either::Right(false));
    /// ```
    #[cfg(feature = "either")]
    pub fn into_either(self) -> either::Either<L, R> {
        match self {
            Self::Left(l) => either::Either::Left(l),
            Self::Right(r) => either::Either::Right(r),
        }
    }
}

impl<T> AxumEither<T, T> {
    /// Extract the inner value if `L` and `R` are the same type
    ///
    /// # Examples
    /// ```
    /// # use axum_either::AxumEither;
    /// let l: AxumEither<i32, bool> = AxumEither::Left(10);
    /// let l = l.map_lr(|i| i * 10, |_b| unreachable!());
    /// assert_eq!(l.into_inner(), 100);
    /// ```
    pub fn into_inner(self) -> T {
        match self {
            Self::Left(l) => l,
            Self::Right(r) => r,
        }
    }
}

#[async_trait::async_trait]
impl<L, R, B> FromRequest<B> for AxumEither<L, R>
where
    L: FromRequest<B>,
    L::Rejection: Send,
    R: FromRequest<B>,
    B: Send,
{
    type Rejection = AxumEitherRejection<L::Rejection, R::Rejection>;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let left_error = match L::from_request(req).await {
            Ok(l) => return Ok(Self::Left(l)),
            Err(e) => e,
        };

        let right_error = match R::from_request(req).await {
            Ok(r) => return Ok(Self::Right(r)),
            Err(e) => e,
        };

        Err(AxumEitherRejection {
            left_error,
            right_error,
        })
    }
}

impl<L, R> IntoResponse for AxumEither<L, R>
where
    L: IntoResponse,
    R: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Self::Left(l) => l.into_response(),
            Self::Right(r) => r.into_response(),
        }
    }
}

/// A rejection when both values of [`AxumEither`] are rejected while parsing.
#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct AxumEitherRejection<LE, RE>
where
    LE: IntoResponse,
    RE: IntoResponse,
{
    /// The error that occured while parsing the left variant
    pub left_error: LE,
    /// The error that occured while parsing the right variant
    pub right_error: RE,
}

impl<LE, RE> IntoResponse for AxumEitherRejection<LE, RE>
where
    LE: IntoResponse,
    RE: IntoResponse,
{
    fn into_response(self) -> Response {
        let left_response = self.left_error.into_response();
        let right_response = self.right_error.into_response();
        let status = if left_response.status().is_server_error()
            || right_response.status().is_server_error()
        {
            StatusCode::INTERNAL_SERVER_ERROR
        } else {
            StatusCode::BAD_REQUEST
        };

        (
            status,
            [(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"))],
            format!(
                "Could not parse request\n\tleft error: {:?}\n\tright error: {:?}",
                left_response, right_response
            ),
        )
            .into_response()
    }
}

#[macro_export]
/// Build a chain of axum eithers which may contain any of the given type.
///
/// # Examples
/// ```
/// # use axum_either::AxumEither;
/// let val: AxumEither<i32, u32> = AxumEither::Left(0);
/// let _val2: axum_either::one_of!(i32, u32) = val;
/// let val: AxumEither<i32, AxumEither<u8, u32>> = AxumEither::Left(0);
/// let _val2: axum_either::one_of!(i32, u8, u32) = val;
/// let val: AxumEither<i32, AxumEither<u8, AxumEither<u16, u32>>> = AxumEither::Left(0);
/// let _val2: axum_either::one_of!(i32, u8, u16, u32) = val;
/// ```
macro_rules! one_of {
    ($t0:ty, $t1:ty) => {
        AxumEither<$t0, $t1>
    };
    ($t0:ty, $($tleft:ty),+) => {
        AxumEither<$t0, $crate::one_of!($($tleft),+)>
    };
}

#[macro_export]
/// Match a chain of [`AxumEither`]s from left to right
///
/// # Examples
/// ```
/// # use axum_either::AxumEither;
/// let either: axum_either::one_of!(i32, u32, bool) = AxumEither::Right(AxumEither::Right(false));
/// axum_either::match_one_of!{either,
///     _val => unreachable!(),
///     _val => unreachable!(),
///     val => assert_eq!(val, false),
/// };
/// ```
macro_rules! match_one_of {
    ($either:expr, $id0:pat => $expr0:expr, $id1:pat => $expr1:expr,) => {
        match $either {
            $crate::AxumEither::Left($id0) => $expr0,
            $crate::AxumEither::Right($id1) => $expr1,
        }
    };
    ($either:expr, $id0:pat => $expr0:expr, $($idleft:pat => $exprleft:expr,)+) => {
        match $either {
            $crate::AxumEither::Left($id0) => $expr0,
            $crate::AxumEither::Right(eithers_left) => {
                $crate::match_one_of!{eithers_left, $($idleft => $exprleft,)+}
            }
        }
    };
}

#[macro_export]
/// Match a chain of [`AxumEither`]s from left to right and map them to an `AxumEither` directly.
///
/// # Examples
/// ```
/// # use axum_either::AxumEither;
/// let either: axum_either::one_of!(i32, u32, u8) = AxumEither::Right(AxumEither::Right(32u8));
/// let either = axum_either::map_one_of!{either,
///     _val => unreachable!(),
///     _val => unreachable!(),
///     val => val + 100,
/// };
/// assert_eq!(either, AxumEither::Right(AxumEither::Right(132u8)));
/// ```
macro_rules! map_one_of {
    ($either:expr, $id0:pat => $expr0:expr, $id1:pat => $expr1:expr,) => {
        match $either {
            $crate::AxumEither::Left($id0) => $crate::AxumEither::Left($expr0),
            $crate::AxumEither::Right($id1) => $crate::AxumEither::Right($expr1),
        }
    };
    ($either:expr, $id0:pat => $expr0:expr, $($idleft:pat => $exprleft:expr,)+) => {
        match $either {
            $crate::AxumEither::Left($id0) => $crate::AxumEither::Left($expr0),
            $crate::AxumEither::Right(eithers_left) => {
                $crate::AxumEither::Right(
                    $crate::map_one_of!{eithers_left, $(($idleft) => $exprleft,)+}
                )
            }
        }
    };
}
