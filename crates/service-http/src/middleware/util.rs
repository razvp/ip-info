use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use core::task::Poll;
use std::future::Future;

pin_project_lite::pin_project! {
    #[project = EnumProj]
    pub enum ResponseFuture<F> {
        Stop {
            response: (StatusCode, &'static str)
        },
        Continue {
            #[pin]
            future: F
        }
    }
}

impl<F, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response, E>>,
{
    type Output = F::Output;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this {
            EnumProj::Stop { response } => Poll::Ready(Ok(response.into_response())),
            EnumProj::Continue { future } => future.poll(cx),
        }
    }
}
