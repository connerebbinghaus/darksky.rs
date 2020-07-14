// ISC License (ISC)
//
// Copyright (c) 2016, Zeyla Hellyer <zey@zey.moe>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER
// RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF
// CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//! Bridged support for the `hyper` library.

use reqwest::Client;
use crate::models::Forecast;
use crate::{internal, utils, Error};
use async_trait::async_trait;

/// The trait for `hyper` implementations to different DarkSky routes.
#[async_trait]
pub trait DarkskyReqwestRequester {
    async fn get_forecast<'a, 'b, T: AsRef<str> + Send>(
        &'a self,
        token: T,
        latitude: f64,
        longitude: f64,
    ) -> Result<Forecast, Error>;
}

#[async_trait]
impl DarkskyReqwestRequester for Client {
    async fn get_forecast<'a, 'b, T: AsRef<str> + Send>(
        &'a self,
        token: T,
        latitude: f64,
        longitude: f64,
    ) -> Result<Forecast, Error> {
        let url = utils::uri(token.as_ref(), latitude, longitude);

        let res = self.get(&url).send().await?;
        let data = res.bytes().await?;

        internal::from_chunk(&data)
    }
}