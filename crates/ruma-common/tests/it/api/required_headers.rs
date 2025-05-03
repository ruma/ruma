use assert_matches2::assert_matches;
use http::header::{CONTENT_DISPOSITION, LOCATION};
use ruma_common::{
    api::{
        error::{
            DeserializationError, FromHttpRequestError, FromHttpResponseError,
            HeaderDeserializationError,
        },
        request, response, IncomingRequest, IncomingResponse, MatrixVersion, Metadata,
        OutgoingRequest, OutgoingResponse, SendAccessToken, SupportedVersions,
    },
    http_headers::{ContentDisposition, ContentDispositionType},
    metadata,
};

const METADATA: Metadata = metadata! {
    method: GET,
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/_matrix/my/endpoint",
    }
};

/// Request type for the `required_headers` endpoint.
#[request]
pub struct Request {
    #[ruma_api(header = LOCATION)]
    pub location: String,
    #[ruma_api(header = CONTENT_DISPOSITION)]
    pub content_disposition: ContentDisposition,
}

/// Response type for the `required_headers` endpoint.
#[response]
pub struct Response {
    #[ruma_api(header = LOCATION)]
    pub stuff: String,
    #[ruma_api(header = CONTENT_DISPOSITION)]
    pub content_disposition: ContentDisposition,
}

#[test]
fn request_serde() {
    let location = "https://other.tld/page/";
    let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
        .with_filename(Some("my_file".to_owned()));
    let req =
        Request { location: location.to_owned(), content_disposition: content_disposition.clone() };
    let supported =
        SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Vec::new() };

    let mut http_req = req
        .clone()
        .try_into_http_request::<Vec<u8>>(
            "https://homeserver.tld",
            SendAccessToken::None,
            &supported,
        )
        .unwrap();
    assert_matches!(http_req.headers().get(LOCATION), Some(_));
    assert_matches!(http_req.headers().get(CONTENT_DISPOSITION), Some(_));

    let req2 = Request::try_from_http_request::<_, &str>(http_req.clone(), &[]).unwrap();
    assert_eq!(req2.location, location);
    assert_eq!(req2.content_disposition, content_disposition);

    // Try removing the headers.
    http_req.headers_mut().remove(LOCATION).unwrap();
    http_req.headers_mut().remove(CONTENT_DISPOSITION).unwrap();

    let err = Request::try_from_http_request::<_, &str>(http_req.clone(), &[]).unwrap_err();
    assert_matches!(
        err,
        FromHttpRequestError::Deserialization(DeserializationError::Header(
            HeaderDeserializationError::MissingHeader(_)
        ))
    );

    // Try setting invalid header.
    http_req.headers_mut().insert(LOCATION, location.try_into().unwrap());
    http_req.headers_mut().insert(CONTENT_DISPOSITION, ";".try_into().unwrap());

    let err = Request::try_from_http_request::<_, &str>(http_req, &[]).unwrap_err();
    assert_matches!(
        err,
        FromHttpRequestError::Deserialization(DeserializationError::Header(
            HeaderDeserializationError::InvalidHeader(_)
        ))
    );
}

#[test]
fn response_serde() {
    let location = "https://other.tld/page/";
    let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
        .with_filename(Some("my_file".to_owned()));
    let res =
        Response { stuff: location.to_owned(), content_disposition: content_disposition.clone() };

    let mut http_res = res.clone().try_into_http_response::<Vec<u8>>().unwrap();
    assert_matches!(http_res.headers().get(LOCATION), Some(_));
    assert_matches!(http_res.headers().get(CONTENT_DISPOSITION), Some(_));

    let res2 = Response::try_from_http_response(http_res.clone()).unwrap();
    assert_eq!(res2.stuff, location);
    assert_eq!(res2.content_disposition, content_disposition);

    // Try removing the headers.
    http_res.headers_mut().remove(LOCATION).unwrap();
    http_res.headers_mut().remove(CONTENT_DISPOSITION).unwrap();

    let err = Response::try_from_http_response(http_res.clone()).unwrap_err();
    assert_matches!(
        err,
        FromHttpResponseError::Deserialization(DeserializationError::Header(
            HeaderDeserializationError::MissingHeader(_)
        ))
    );

    // Try setting invalid header.
    http_res.headers_mut().insert(LOCATION, location.try_into().unwrap());
    http_res.headers_mut().insert(CONTENT_DISPOSITION, ";".try_into().unwrap());

    let err = Response::try_from_http_response(http_res).unwrap_err();
    assert_matches!(
        err,
        FromHttpResponseError::Deserialization(DeserializationError::Header(
            HeaderDeserializationError::InvalidHeader(_)
        ))
    );
}
