# ruma-api-macros

**ruma-api-macros** provides a procedural macro for easily generating [ruma-api]-compatible API endpoints.
You define the endpoint's metadata, request fields, and response fields, and the macro generates all the necessary types and implements all the necessary traits.

[ruma-api]: https://github.com/ruma/ruma/tree/main/ruma-api

## Usage

This crate is not meant to be used directly; instead, you can use it through the re-exports in ruma-api.

Here is an example that shows most of the macro's functionality:

```rust
pub mod some_endpoint {
    use ruma_api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: GET, // An `http::Method` constant. No imports required.
            name: "some_endpoint",
            path: "/_matrix/some/endpoint/:baz", // Variable path components start with a colon.
            rate_limited: false,
            authentication: None,
        }

        request: {
            // With no attribute on the field, it will be put into the body of the request.
            pub foo: String,

            // This value will be put into the "Content-Type" HTTP header.
            #[ruma_api(header = CONTENT_TYPE)]
            pub content_type: String

            // This value will be put into the query string of the request's URL.
            #[ruma_api(query)]
            pub bar: String,

            // This value will be inserted into the request's URL in place of the
            // ":baz" path component.
            #[ruma_api(path)]
            pub baz: String,
        }

        response: {
            // This value will be extracted from the "Content-Type" HTTP header.
            #[ruma_api(header = CONTENT_TYPE)]
            pub content_type: String

            // With no attribute on the field, it will be extracted from the body of the response.
            pub value: String,
        }

        // An error can also be specified or defaults to `ruma_api::error::Void`.
        error: ruma_api::Error
    }
}
```
