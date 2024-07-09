use std::{net::Ipv4Addr, str::FromStr};

use axum::{
    extract::Request,
    http::{HeaderName, StatusCode},
    response::Response,
};
use tower::{Layer, Service};

use super::util::ResponseFuture;
use crate::ClientIp;

#[derive(Clone)]
pub struct EnsureReverseProxyLayer {
    header: &'static HeaderName,
}

impl EnsureReverseProxyLayer {
    pub fn new(header: impl AsRef<str>) -> Self {
        let header = HeaderName::from_str(header.as_ref()).unwrap();
        let header: &'static HeaderName = Box::leak(Box::new(header));
        Self { header }
    }
}

impl<S> Layer<S> for EnsureReverseProxyLayer {
    type Service = EnsureReverseProxy<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EnsureReverseProxy {
            inner,
            header: self.header,
        }
    }
}

#[derive(Clone)]
pub struct EnsureReverseProxy<S> {
    inner: S,
    header: &'static HeaderName,
}

impl<S> Service<Request> for EnsureReverseProxy<S>
where
    S: Service<Request, Response = Response>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        match set_ip_extension_or_return_error_response(&mut req, self.header) {
            // ClientIp has been set as extension on the request
            // and the request is passed to the next layer
            Ok(_) => ResponseFuture::Continue {
                future: self.inner.call(req),
            },
            // Couldn't find one valid IP set by the reverse proxy
            // so the requests stops at this layer and we respond to the client
            Err(response) => ResponseFuture::Stop { response },
        }
    }
}

fn set_ip_extension_or_return_error_response(
    req: &mut Request,
    header: &HeaderName,
) -> Result<(), (StatusCode, &'static str)> {
    let ips: Result<Vec<_>, _> = req
        .headers()
        .get_all(header)
        .iter()
        .map(|hv| {
            let ip = hv
                .to_str()
                .map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        "Value in reverse proxy header is not UTF-8",
                    )
                })?
                .parse::<Ipv4Addr>()
                .map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        "Value in reverse proxy header is not an IPv4",
                    )
                })?;
            Ok::<Ipv4Addr, (StatusCode, &'static str)>(ip)
        })
        .collect();
    let ips = ips?;

    match ips.len() {
        1 => {
            req.extensions_mut().insert(ClientIp(ips[0]));
            Ok(())
        }
        0 => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Expected reverse proxy header",
        )),
        _ => Err((
            StatusCode::BAD_REQUEST,
            "More than one IP in reverse proxy header",
        )),
    }
}
