use assert_matches2::assert_matches;
use http::header::{CONTENT_DISPOSITION, LOCATION};
use ruma_common::{
    api::{
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

/// Request type for the `optional_headers` endpoint.
#[request]
pub struct Request {
    #[ruma_api(header = LOCATION)]
    pub location: Option<String>,
    #[ruma_api(header = CONTENT_DISPOSITION)]
    pub content_disposition: Option<ContentDisposition>,
}

/// Response type for the `optional_headers` endpoint.
#[response]
pub struct Response {
    #[ruma_api(header = LOCATION)]
    pub stuff: Option<String>,
    #[ruma_api(header = CONTENT_DISPOSITION)]
    pub content_disposition: Option<ContentDisposition>,
}

#[test]
fn request_serde_no_header() {
    let req = Request { location: None, content_disposition: None };
    let supported =
        SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Vec::new() };

    let http_req = req
        .clone()
        .try_into_http_request::<Vec<u8>>(
            "https://homeserver.tld",
            SendAccessToken::None,
            &supported,
        )
        .unwrap();
    assert_matches!(http_req.headers().get(LOCATION), None);
    assert_matches!(http_req.headers().get(CONTENT_DISPOSITION), None);

    let req2 = Request::try_from_http_request::<_, &str>(http_req, &[]).unwrap();
    assert_eq!(req2.location, None);
    assert_eq!(req2.content_disposition, None);
}

#[test]
fn request_serde_with_header() {
    let location = "https://other.tld/page/";
    let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
        .with_filename(Some("my_file".to_owned()));
    let req = Request {
        location: Some(location.to_owned()),
        content_disposition: Some(content_disposition.clone()),
    };
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
    assert_eq!(req2.location.unwrap(), location);
    assert_eq!(req2.content_disposition.unwrap(), content_disposition);

    // Try removing the headers.
    http_req.headers_mut().remove(LOCATION).unwrap();
    http_req.headers_mut().remove(CONTENT_DISPOSITION).unwrap();

    let req3 = Request::try_from_http_request::<_, &str>(http_req.clone(), &[]).unwrap();
    assert_eq!(req3.location, None);
    assert_eq!(req3.content_disposition, None);

    // Try setting invalid header.
    http_req.headers_mut().insert(CONTENT_DISPOSITION, ";".try_into().unwrap());

    let req4 = Request::try_from_http_request::<_, &str>(http_req, &[]).unwrap();
    assert_eq!(req4.location, None);
    assert_eq!(req4.content_disposition, None);
}

#[test]
fn response_serde_no_header() {
    let res = Response { stuff: None, content_disposition: None };

    let http_res = res.clone().try_into_http_response::<Vec<u8>>().unwrap();
    assert_matches!(http_res.headers().get(LOCATION), None);
    assert_matches!(http_res.headers().get(CONTENT_DISPOSITION), None);

    let res2 = Response::try_from_http_response(http_res).unwrap();
    assert_eq!(res2.stuff, None);
    assert_eq!(res2.content_disposition, None);
}

#[test]
fn response_serde_with_header() {
    let location = "https://other.tld/page/";
    let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
        .with_filename(Some("my_file".to_owned()));
    let res = Response {
        stuff: Some(location.to_owned()),
        content_disposition: Some(content_disposition.clone()),
    };

    let mut http_res = res.clone().try_into_http_response::<Vec<u8>>().unwrap();
    assert_matches!(http_res.headers().get(LOCATION), Some(_));
    assert_matches!(http_res.headers().get(CONTENT_DISPOSITION), Some(_));

    let res2 = Response::try_from_http_response(http_res.clone()).unwrap();
    assert_eq!(res2.stuff.unwrap(), location);
    assert_eq!(res2.content_disposition.unwrap(), content_disposition);

    // Try removing the headers.
    http_res.headers_mut().remove(LOCATION).unwrap();
    http_res.headers_mut().remove(CONTENT_DISPOSITION).unwrap();

    let res3 = Response::try_from_http_response(http_res.clone()).unwrap();
    assert_eq!(res3.stuff, None);
    assert_eq!(res3.content_disposition, None);

    // Try setting invalid header.
    http_res.headers_mut().insert(CONTENT_DISPOSITION, ";".try_into().unwrap());

    let res4 = Response::try_from_http_response(http_res).unwrap();
    assert_eq!(res4.stuff, None);
    assert_eq!(res4.content_disposition, None);
}
