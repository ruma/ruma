//! Authenticated endpoints for the content repository, according to [MSC3916].
//!
//! [MSC3916]: https://github.com/matrix-org/matrix-spec-proposals/pull/3916

use ruma_common::http_headers::ContentDisposition;
use serde::{Deserialize, Serialize};

pub mod get_content;
pub mod get_content_thumbnail;

/// The `multipart/mixed` mime "essence".
const MULTIPART_MIXED: &str = "multipart/mixed";
/// The maximum number of headers to parse in a body part.
const MAX_HEADERS_COUNT: usize = 32;
/// The length of the generated boundary.
const GENERATED_BOUNDARY_LENGTH: usize = 30;

/// The metadata of a file from the content repository.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ContentMetadata {}

impl ContentMetadata {
    /// Creates a new empty `ContentMetadata`.
    pub fn new() -> Self {
        Self {}
    }
}

/// A file from the content repository or the location where it can be found.
#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum FileOrLocation {
    /// The content of the file.
    File(Content),

    /// The file is at the given URL.
    Location(String),
}

/// The content of a file from the content repository.
#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Content {
    /// The content of the file as bytes.
    pub file: Vec<u8>,

    /// The content type of the file that was previously uploaded.
    pub content_type: Option<String>,

    /// The value of the `Content-Disposition` HTTP header, possibly containing the name of the
    /// file that was previously uploaded.
    pub content_disposition: Option<ContentDisposition>,
}

impl Content {
    /// Creates a new `Content` with the given bytes.
    pub fn new(file: Vec<u8>) -> Self {
        Self { file, content_type: None, content_disposition: None }
    }
}

/// Serialize the given metadata and content into a `http::Response` `multipart/mixed` body.
///
/// Returns a tuple containing the boundary used
#[cfg(feature = "server")]
fn try_into_multipart_mixed_response<T: Default + bytes::BufMut>(
    metadata: &ContentMetadata,
    content: &FileOrLocation,
) -> Result<http::Response<T>, ruma_common::api::error::IntoHttpError> {
    use std::io::Write as _;

    use rand::Rng as _;

    let boundary = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .map(char::from)
        .take(GENERATED_BOUNDARY_LENGTH)
        .collect::<String>();

    let mut body_writer = T::default().writer();

    // Add first boundary separator and header for the metadata.
    let _ = write!(
        body_writer,
        "\r\n--{boundary}\r\n{}: {}\r\n\r\n",
        http::header::CONTENT_TYPE,
        mime::APPLICATION_JSON
    );

    // Add serialized metadata.
    serde_json::to_writer(&mut body_writer, metadata)?;

    // Add second boundary separator.
    let _ = write!(body_writer, "\r\n--{boundary}\r\n");

    // Add content.
    match content {
        FileOrLocation::File(content) => {
            // Add headers.
            let content_type =
                content.content_type.as_deref().unwrap_or(mime::APPLICATION_OCTET_STREAM.as_ref());
            let _ = write!(body_writer, "{}: {content_type}\r\n", http::header::CONTENT_TYPE);

            if let Some(content_disposition) = &content.content_disposition {
                let _ = write!(
                    body_writer,
                    "{}: {content_disposition}\r\n",
                    http::header::CONTENT_DISPOSITION
                );
            }

            // Add empty line separator after headers.
            let _ = body_writer.write_all(b"\r\n");

            // Add bytes.
            let _ = body_writer.write_all(&content.file);
        }
        FileOrLocation::Location(location) => {
            // Only add location header and empty line separator.
            let _ = write!(body_writer, "{}: {location}\r\n\r\n", http::header::LOCATION);
        }
    }

    // Add final boundary.
    let _ = write!(body_writer, "\r\n--{boundary}--");

    let content_type = format!("{MULTIPART_MIXED}; boundary={boundary}");
    let body = body_writer.into_inner();

    Ok(http::Response::builder().header(http::header::CONTENT_TYPE, content_type).body(body)?)
}

/// Deserialize the given metadata and content from a `http::Response` with a `multipart/mixed`
/// body.
#[cfg(feature = "client")]
fn try_from_multipart_mixed_response<T: AsRef<[u8]>>(
    http_response: http::Response<T>,
) -> Result<
    (ContentMetadata, FileOrLocation),
    ruma_common::api::error::FromHttpResponseError<ruma_common::api::error::MatrixError>,
> {
    use ruma_common::api::error::{HeaderDeserializationError, MultipartMixedDeserializationError};

    // First, get the boundary from the content type header.
    let body_content_type = http_response
        .headers()
        .get(http::header::CONTENT_TYPE)
        .ok_or_else(|| HeaderDeserializationError::MissingHeader("Content-Type".to_owned()))?
        .to_str()?
        .parse::<mime::Mime>()
        .map_err(|e| HeaderDeserializationError::InvalidHeader(e.into()))?;

    if !body_content_type.essence_str().eq_ignore_ascii_case(MULTIPART_MIXED) {
        return Err(HeaderDeserializationError::InvalidHeaderValue {
            header: "Content-Type".to_owned(),
            expected: MULTIPART_MIXED.to_owned(),
            unexpected: body_content_type.essence_str().to_owned(),
        }
        .into());
    }

    let boundary = body_content_type
        .get_param("boundary")
        .ok_or(HeaderDeserializationError::MissingMultipartBoundary)?
        .as_str()
        .as_bytes();

    // Split the body with the boundary.
    let body = http_response.body().as_ref();

    let mut full_boundary = Vec::with_capacity(boundary.len() + 4);
    full_boundary.extend_from_slice(b"\r\n--");
    full_boundary.extend_from_slice(boundary);

    let mut boundaries = memchr::memmem::find_iter(body, &full_boundary);

    let metadata_start = boundaries.next().ok_or_else(|| {
        MultipartMixedDeserializationError::MissingBodyParts { expected: 2, found: 0 }
    })? + full_boundary.len();
    let metadata_end = boundaries.next().ok_or_else(|| {
        MultipartMixedDeserializationError::MissingBodyParts { expected: 2, found: 0 }
    })?;

    let (_raw_metadata_headers, serialized_metadata) =
        parse_multipart_body_part(body, metadata_start, metadata_end)?;

    // Don't search for anything in the headers, just deserialize the content that should be JSON.
    let metadata = serde_json::from_slice(serialized_metadata)?;

    // Look at the part containing the media content now.
    let content_start = metadata_end + full_boundary.len();
    let content_end = boundaries.next().ok_or_else(|| {
        MultipartMixedDeserializationError::MissingBodyParts { expected: 2, found: 1 }
    })?;

    let (raw_content_headers, file) = parse_multipart_body_part(body, content_start, content_end)?;

    // Parse the headers to retrieve the content type and content disposition.
    let mut content_headers = [httparse::EMPTY_HEADER; MAX_HEADERS_COUNT];
    httparse::parse_headers(raw_content_headers, &mut content_headers)
        .map_err(|e| MultipartMixedDeserializationError::InvalidHeader(e.into()))?;

    let mut location = None;
    let mut content_type = None;
    let mut content_disposition = None;
    for header in content_headers {
        if header.name.is_empty() {
            // This is a empty header, we have reached the end of the parsed headers.
            break;
        }

        if header.name == http::header::LOCATION {
            location = Some(
                String::from_utf8(header.value.to_vec())
                    .map_err(|e| MultipartMixedDeserializationError::InvalidHeader(e.into()))?,
            );

            // This is the only header we need, stop parsing.
            break;
        } else if header.name == http::header::CONTENT_TYPE {
            content_type = Some(
                String::from_utf8(header.value.to_vec())
                    .map_err(|e| MultipartMixedDeserializationError::InvalidHeader(e.into()))?,
            );
        } else if header.name == http::header::CONTENT_DISPOSITION {
            content_disposition = Some(
                ContentDisposition::try_from(header.value)
                    .map_err(|e| MultipartMixedDeserializationError::InvalidHeader(e.into()))?,
            );
        }
    }

    let content = if let Some(location) = location {
        FileOrLocation::Location(location)
    } else {
        FileOrLocation::File(Content { file: file.to_owned(), content_type, content_disposition })
    };

    Ok((metadata, content))
}

/// Parse the multipart body part in the given bytes, starting and ending at the given positions.
///
/// Returns a `(headers_bytes, content_bytes)` tuple. Returns an error if the separation between the
/// headers and the content could not be found.
#[cfg(feature = "client")]
fn parse_multipart_body_part(
    bytes: &[u8],
    start: usize,
    end: usize,
) -> Result<(&[u8], &[u8]), ruma_common::api::error::MultipartMixedDeserializationError> {
    use ruma_common::api::error::MultipartMixedDeserializationError;

    // The part should start with a newline after the boundary. We need to ignore characters before
    // it in case of extra whitespaces, and for compatibility it might not have a CR.
    let headers_start = memchr::memchr(b'\n', &bytes[start..end])
        .expect("the end boundary contains a newline")
        + start
        + 1;

    // Let's find an empty line now.
    let mut line_start = headers_start;
    let mut line_end;

    loop {
        line_end = memchr::memchr(b'\n', &bytes[line_start..end])
            .ok_or(MultipartMixedDeserializationError::MissingBodyPartInnerSeparator)?
            + line_start
            + 1;

        if matches!(&bytes[line_start..line_end], b"\r\n" | b"\n") {
            break;
        }

        line_start = line_end;
    }

    Ok((&bytes[headers_start..line_start], &bytes[line_end..end]))
}

#[cfg(all(test, feature = "client", feature = "server"))]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::http_headers::{ContentDisposition, ContentDispositionType};

    use super::{
        try_from_multipart_mixed_response, try_into_multipart_mixed_response, Content,
        ContentMetadata, FileOrLocation,
    };

    #[test]
    fn multipart_mixed_content_ascii_filename_conversions() {
        let file = "s⌽me UTF-8 Ťext".as_bytes();
        let content_type = "text/plain";
        let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
            .with_filename(Some("filename.txt".to_owned()));

        let outgoing_metadata = ContentMetadata::new();
        let outgoing_content = FileOrLocation::File(Content {
            file: file.to_vec(),
            content_type: Some(content_type.to_owned()),
            content_disposition: Some(content_disposition.clone()),
        });

        let response =
            try_into_multipart_mixed_response::<Vec<u8>>(&outgoing_metadata, &outgoing_content)
                .unwrap();

        let (_incoming_metadata, incoming_content) =
            try_from_multipart_mixed_response(response).unwrap();

        assert_matches!(incoming_content, FileOrLocation::File(incoming_content));
        assert_eq!(incoming_content.file, file);
        assert_eq!(incoming_content.content_type.unwrap(), content_type);
        assert_eq!(incoming_content.content_disposition, Some(content_disposition));
    }

    #[test]
    fn multipart_mixed_content_utf8_filename_conversions() {
        let file = "s⌽me UTF-8 Ťext".as_bytes();
        let content_type = "text/plain";
        let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
            .with_filename(Some("fȈlƩnąmǝ.txt".to_owned()));

        let outgoing_metadata = ContentMetadata::new();
        let outgoing_content = FileOrLocation::File(Content {
            file: file.to_vec(),
            content_type: Some(content_type.to_owned()),
            content_disposition: Some(content_disposition.clone()),
        });

        let response =
            try_into_multipart_mixed_response::<Vec<u8>>(&outgoing_metadata, &outgoing_content)
                .unwrap();

        let (_incoming_metadata, incoming_content) =
            try_from_multipart_mixed_response(response).unwrap();

        assert_matches!(incoming_content, FileOrLocation::File(incoming_content));
        assert_eq!(incoming_content.file, file);
        assert_eq!(incoming_content.content_type.unwrap(), content_type);
        assert_eq!(incoming_content.content_disposition, Some(content_disposition));
    }

    #[test]
    fn multipart_mixed_location_conversions() {
        let location = "https://server.local/media/filename.txt";

        let outgoing_metadata = ContentMetadata::new();
        let outgoing_content = FileOrLocation::Location(location.to_owned());

        let response =
            try_into_multipart_mixed_response::<Vec<u8>>(&outgoing_metadata, &outgoing_content)
                .unwrap();

        let (_incoming_metadata, incoming_content) =
            try_from_multipart_mixed_response(response).unwrap();

        assert_matches!(incoming_content, FileOrLocation::Location(incoming_location));
        assert_eq!(incoming_location, location);
    }

    #[test]
    fn multipart_mixed_deserialize_invalid() {
        // Missing boundary in headers.
        let body = "\r\n--abcdef\r\n\r\n{}\r\n--abcdef\r\nContent-Type: text/plain\r\n\r\nsome plain text\r\n--abcdef--";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed")
            .body(body)
            .unwrap();

        try_from_multipart_mixed_response(response).unwrap_err();

        // Wrong boundary.
        let body =
            "\r\n--abcdef\r\n\r\n{}\r\n--abcdef\r\nContent-Type: text/plain\r\n\r\nsome plain text\r\n--abcdef--";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed; boundary=012345")
            .body(body)
            .unwrap();

        try_from_multipart_mixed_response(response).unwrap_err();

        // Missing boundary in body.
        let body =
            "\r\n--abcdef\r\n\r\n{}\r\n--abcdef\r\nContent-Type: text/plain\r\n\r\nsome plain text";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed; boundary=abcdef")
            .body(body)
            .unwrap();

        try_from_multipart_mixed_response(response).unwrap_err();

        // Missing header and content empty line separator in body part.
        let body =
            "\r\n--abcdef\r\n{}\r\n--abcdef\r\nContent-Type: text/plain\r\n\r\nsome plain text\r\n--abcdef--";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed; boundary=abcdef")
            .body(body)
            .unwrap();

        try_from_multipart_mixed_response(response).unwrap_err();

        // Control character in header.
        let body =
            "\r\n--abcdef\r\n\r\n{}\r\n--abcdef\r\nContent-Type: text/plain\r\nContent-Disposition: inline; filename=\"my\nfile\"\r\nsome plain text\r\n--abcdef--";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed; boundary=abcdef")
            .body(body)
            .unwrap();

        try_from_multipart_mixed_response(response).unwrap_err();
    }

    #[test]
    fn multipart_mixed_deserialize_valid() {
        // Simple.
        let body =
            "\r\n--abcdef\r\ncontent-type: application/json\r\n\r\n{}\r\n--abcdef\r\ncontent-type: text/plain\r\n\r\nsome plain text\r\n--abcdef--";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed; boundary=abcdef")
            .body(body)
            .unwrap();

        let (_metadata, content) = try_from_multipart_mixed_response(response).unwrap();

        assert_matches!(content, FileOrLocation::File(file_content));
        assert_eq!(file_content.file, b"some plain text");
        assert_eq!(file_content.content_type.unwrap(), "text/plain");
        assert_eq!(file_content.content_disposition, None);

        // Case-insensitive headers.
        let body =
            "\r\n--abcdef\r\nCONTENT-type: application/json\r\n\r\n{}\r\n--abcdef\r\nCONTENT-TYPE: text/plain\r\ncoNtenT-disPosItioN: attachment; filename=my_file.txt\r\n\r\nsome plain text\r\n--abcdef--";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed; boundary=abcdef")
            .body(body)
            .unwrap();

        let (_metadata, content) = try_from_multipart_mixed_response(response).unwrap();

        assert_matches!(content, FileOrLocation::File(file_content));
        assert_eq!(file_content.file, b"some plain text");
        assert_eq!(file_content.content_type.unwrap(), "text/plain");
        let content_disposition = file_content.content_disposition.unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename.unwrap(), "my_file.txt");

        // Extra whitespace.
        let body =
            "   \r\n--abcdef\r\ncontent-type:   application/json   \r\n\r\n {} \r\n--abcdef\r\ncontent-type: text/plain  \r\n\r\nsome plain text\r\n--abcdef--  ";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed; boundary=abcdef")
            .body(body)
            .unwrap();

        let (_metadata, content) = try_from_multipart_mixed_response(response).unwrap();

        assert_matches!(content, FileOrLocation::File(file_content));
        assert_eq!(file_content.file, b"some plain text");
        assert_eq!(file_content.content_type.unwrap(), "text/plain");
        assert_eq!(file_content.content_disposition, None);

        // Missing CR except in boundaries.
        let body =
            "\r\n--abcdef\ncontent-type: application/json\n\n{}\r\n--abcdef\ncontent-type: text/plain  \n\nsome plain text\r\n--abcdef--";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed; boundary=abcdef")
            .body(body)
            .unwrap();

        let (_metadata, content) = try_from_multipart_mixed_response(response).unwrap();

        assert_matches!(content, FileOrLocation::File(file_content));
        assert_eq!(file_content.file, b"some plain text");
        assert_eq!(file_content.content_type.unwrap(), "text/plain");
        assert_eq!(file_content.content_disposition, None);

        // No body part headers.
        let body = "\r\n--abcdef\r\n\r\n{}\r\n--abcdef\r\n\r\nsome plain text\r\n--abcdef--";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed; boundary=abcdef")
            .body(body)
            .unwrap();

        let (_metadata, content) = try_from_multipart_mixed_response(response).unwrap();

        assert_matches!(content, FileOrLocation::File(file_content));
        assert_eq!(file_content.file, b"some plain text");
        assert_eq!(file_content.content_type, None);
        assert_eq!(file_content.content_disposition, None);

        // Raw UTF-8 filename (some kind of compatibility with multipart/form-data).
        let body =
            "\r\n--abcdef\r\ncontent-type: application/json\r\n\r\n{}\r\n--abcdef\r\ncontent-type: text/plain\r\ncontent-disposition: inline; filename=\"ȵ⌾Ⱦԩ💈Ňɠ\"\r\n\r\nsome plain text\r\n--abcdef--";
        let response = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "multipart/mixed; boundary=abcdef")
            .body(body)
            .unwrap();

        let (_metadata, content) = try_from_multipart_mixed_response(response).unwrap();

        assert_matches!(content, FileOrLocation::File(file_content));
        assert_eq!(file_content.file, b"some plain text");
        assert_eq!(file_content.content_type.unwrap(), "text/plain");
        let content_disposition = file_content.content_disposition.unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Inline);
        assert_eq!(content_disposition.filename.unwrap(), "ȵ⌾Ⱦԩ💈Ňɠ");
    }
}
