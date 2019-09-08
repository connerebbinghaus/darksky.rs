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

use futures::Stream;
use hyper::{
    body::Payload,
    client::{connect::Connect, Client},
};
use crate::models::Forecast;
use std::str::FromStr;
use crate::{internal, utils, Error};
use async_trait::async_trait;

/// The trait for `hyper` implementations to different DarkSky routes.
#[async_trait]
pub trait DarkskyHyperRequester {
    /// Retrieve a [forecast][`Forecast`] for the given latitude and longitude.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// extern crate darksky;
    /// extern crate futures;
    /// extern crate hyper;
    /// extern crate hyper_tls;
    /// extern crate tokio_core;
    ///
    /// # use std::error::Error;
    /// #
    /// use darksky::{DarkskyHyperRequester, Block};
    /// use futures::Future;
    /// use hyper::{Body, client::{Client, HttpConnector}};
    /// use hyper_tls::HttpsConnector;
    /// use std::env;
    /// use tokio_core::reactor::Core;
    ///
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// let core = Core::new()?;
    /// let client = Client::builder()
    ///     .build::<_, Body>(HttpsConnector::new(4).unwrap());
    ///
    /// let token = env::var("FORECAST_TOKEN")?;
    /// let lat = 37.8267;
    /// let long = -122.423;
    ///
    /// // We're waiting in this example, but you shouldn't in your code.
    /// match client.get_forecast(&token, lat, long).wait() {
    ///     Ok(forecast) => println!("{:?}", forecast),
    ///     Err(why) => println!("Error getting forecast: {:?}", why),
    /// }
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// [`Forecast`]: ../../models/struct.Forecast.html
    async fn get_forecast<'a, 'b, T: AsRef<str> + Send>(
        &'a self,
        token: T,
        latitude: f64,
        longitude: f64,
    ) -> Result<Forecast, Error>;
}

#[async_trait]
impl<B, C> DarkskyHyperRequester for Client<C, B>
where
    C: Connect + Sync + 'static,
    C::Transport: 'static,
    C::Future: 'static,
    B: Payload + Send + 'static + Default + Stream + Unpin,
    B::Data: Send + Unpin,
    B::Item: AsRef<[u8]>,
{
    async fn get_forecast<'a, 'b, T: AsRef<str> + Send>(
        &'a self,
        token: T,
        latitude: f64,
        longitude: f64,
    ) -> Result<Forecast, Error> {
        let url = utils::uri(token.as_ref(), latitude, longitude);
        let uri = match hyper::Uri::from_str(&url) {
            Ok(v) => v,
            Err(why) => return Err(Error::Uri(why)),
        };

        let res = self.get(uri).await?;
        let mut bod = res.into_body();
        let mut data = vec!();

        while let Some(c) = bod.next().await {
            data.extend_from_slice(&c?);
        }

        internal::from_chunk(&data)
    }
}