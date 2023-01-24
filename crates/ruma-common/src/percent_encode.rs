use percent_encoding::{AsciiSet, CONTROLS};

/// The [path percent-encode set] as defined in the WHATWG URL standard + `/` since
/// we always encode single segments of the path.
///
/// [path percent-encode set]: https://url.spec.whatwg.org/#path-percent-encode-set
pub(crate) const PATH_PERCENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'/');
