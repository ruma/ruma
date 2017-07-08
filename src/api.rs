/// Endpoints that cannot change with new versions of the Matrix specification.
pub mod unversioned {
    /// [GET /_matrix/client/versions](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-versions)
    pub mod get_supported_versions {
        use futures::Future;
        use hyper::client::Connect;
        use ruma_client_api::unversioned::get_supported_versions::Endpoint;
        pub use ruma_client_api::unversioned::get_supported_versions::{Request, Response};

        use {Client, Error};

        /// Make a request to this API endpoint.
        pub fn call<'a, C>(
            client: &'a Client<C>,
            request: Request,
        ) -> impl Future<Item = Response, Error = Error> + 'a
        where
            C: Connect,
        {
            client.request::<Endpoint>(request)
        }
    }
}
