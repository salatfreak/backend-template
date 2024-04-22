use std::{fmt, ops::Deref};

use rocket::{data::{FromData, Outcome}, http::Status, Data, Request};
use sha2::{Sha512, Digest};

/// Proof of work mask with 16 leading one bits
const MASK: u32 = 0u32.wrapping_sub(1) << (32 - 16);

#[derive(Debug, thiserror::Error)]
#[error("invalid proof of work")]
pub enum Error<'r, T: FromData<'r>> {
    #[error("request body is too long")]
    TooLong,

    #[error("invalid proof of work")]
    POW,

    #[error("inner error")]
    Inner(T::Error),
}

#[derive(Clone, Debug)]
pub struct POW<T>(pub T);

impl<T> Deref for POW<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r, T: FromData<'r> + fmt::Debug> FromData<'r> for POW<T> {
    type Error = Error<'r, T>;

    async fn from_data(
        req: &'r Request<'_>, mut data: Data<'r>,
    ) -> Outcome<'r, Self> {
        // handle too long request bodies
        data.peek(512).await;
        if !data.peek_complete() {
            return Outcome::Error((Status::PayloadTooLarge, Error::TooLong));
        }

        // check proof of work
        if !check(data.peek(512).await) {
            return Outcome::Error((Status::PaymentRequired, Error::POW));
        }

        // wrap inner type
        T::from_data(req, data).await.map(|v| Self(v))
            .map_error(|(s, e)| (s, Error::Inner(e)))
    }
}

/// Check proof of work for bytes
fn check(data: &[u8]) -> bool {
    // calculate SHA512 hash
    let mut hasher = Sha512::new();
    hasher.update(data);
    let hash = hasher.finalize();

    // interprete the first 4 bytes as big-endian unsigned integer
    let num: u32 = (0..=3).map(|i| (hash[3 - i] as u32) << (i * 8)).sum();

    // check if all non-masked bits are zero
    num & MASK == 0
}
