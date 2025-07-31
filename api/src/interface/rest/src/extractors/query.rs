use std::ops::{Deref, DerefMut};

use axum::{
    extract::{FromRequestParts, Query},
    http,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::response::ApiError;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ValidQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ApiError;

    fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async {
            let Query(value) = Query::<T>::from_request_parts(parts, state).await?;
            value.validate()?;
            Ok(Self(value))
        }
    }
}

impl<T> Deref for ValidQuery<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ValidQuery<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
