use http::Request;
use percent_encoding::{percent_encode, AsciiSet, CONTROLS};
use std::borrow::Cow;
use std::convert::TryInto;

/// A `tower` middleware Layer that strips out empty parameters.
#[derive(Clone)]
pub struct Layer;

impl<S> tower::Layer<S> for Layer {
    type Service = Service<S>;
    fn layer(&self, s: S) -> Self::Service {
        Service { s }
    }
}

#[derive(Clone)]
pub struct Service<S> {
    s: S,
}

impl<S, R, B> tower::Service<Request<B>> for Service<S>
where
    S: tower::Service<Request<B>, Response = R> + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.s.poll_ready(cx)
    }

    fn call(&mut self, mut r: Request<B>) -> Self::Future {
        let uri = r.uri();
        let new_query = (|| {
            if let Some(query) = uri.query() {
                if let Cow::Owned(n) = remove_empty_query_params(query) {
                    return Some(n);
                }
            }
            None
        })();

        if let Some(nq) = new_query {
            let mut parts = uri.clone().into_parts();
            let new_path_and_query = format!("{path}?{query}", path = uri.path(), query = nq);
            parts.path_and_query = Some(new_path_and_query.try_into().unwrap());

            *r.uri_mut() = http::uri::Uri::from_parts(parts).unwrap();
        }

        self.s.call(r)
    }
}

fn needs_modification(q: &str) -> bool {
    !form_urlencoded::parse(q.as_bytes())
        .all(|(k, v)| matches!(k, Cow::Borrowed(_)) && matches!(k, Cow::Borrowed(_)) && v.len() > 0)
}

/// Returns `query` or a version of `query` where all empty parameters are removed.
const QUERY_SET: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'#').add(b'<').add(b'>');

pub fn remove_empty_query_params(q: &str) -> Cow<'_, str> {
    if !needs_modification(q) {
        Cow::Borrowed(q)
    } else {
        Cow::Owned(
            form_urlencoded::parse(q.as_bytes())
                .filter(|(_, v)| v.len() > 0)
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        percent_encode(k.as_bytes(), QUERY_SET).to_string(),
                        percent_encode(v.as_bytes(), QUERY_SET).to_string()
                    )
                })
                .collect::<Vec<_>>()
                .as_slice()
                .join("&"),
        )
    }
}

#[test]
fn test_remove_empty_query_params() {
    assert_eq!(
        Cow::<str>::Owned("x=1&y=2".to_string()),
        remove_empty_query_params("x=1&y=2&z=")
    );

    {
        let s = "x=1&y=2";
        assert_eq!(Cow::<str>::Borrowed(s), remove_empty_query_params(s));
    }
}
