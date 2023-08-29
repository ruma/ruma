use subslice::SubsliceExt as _;

#[derive(Debug)]
pub(super) enum CompatibleDocument<'a> {
    WellFormed(&'a [u8]),
    CleanedFromRing(Vec<u8>),
}

impl<'a> CompatibleDocument<'a> {
    pub(super) fn from_bytes(bytes: &'a [u8]) -> Self {
        if is_ring(bytes) {
            Self::CleanedFromRing(fix_ring_doc(bytes.to_vec()))
        } else {
            Self::WellFormed(bytes)
        }
    }
}

// Ring uses a very specific template to generate its documents,
// and so this is essentially a sentinel value of that.
//
// It corresponds to CONTEXT-SPECIFIC[1](35) { BIT-STRING(32) {...} } in ASN.1
//
// A well-formed bit would look like just CONTEXT-SPECIFIC[1](32) { ... }
//
// Note: this is purely a sentinel value, don't take these bytes out of context
// to detect or fiddle with the document.
const RING_TEMPLATE_CONTEXT_SPECIFIC: &[u8] = &[0xA1, 0x23, 0x03, 0x21];

// A checked well-formed context-specific[1] prefix.
const WELL_FORMED_CONTEXT_ONE_PREFIX: &[u8] = &[0x81, 0x21];

// If present, removes a malfunctioning pubkey suffix and adjusts the length at the start.
fn fix_ring_doc(mut doc: Vec<u8>) -> Vec<u8> {
    assert!(!doc.is_empty());
    // Check if first tag is ASN.1 SEQUENCE
    assert_eq!(doc[0], 0x30);
    // Second byte asserts the length for the rest of the document
    assert_eq!(doc[1] as usize, doc.len() - 2);

    let idx = doc
        .find(RING_TEMPLATE_CONTEXT_SPECIFIC)
        .expect("Expected to find ring template in doc, but found none.");

    // Snip off the malformed bit.
    let suffix = doc.split_off(idx);

    // Feed back an actual well-formed prefix.
    doc.extend(WELL_FORMED_CONTEXT_ONE_PREFIX);

    // Then give it the actual public key.
    doc.extend(&suffix[4..]);

    doc[1] = doc.len() as u8 - 2;

    doc
}

fn is_ring(bytes: &[u8]) -> bool {
    bytes.find(RING_TEMPLATE_CONTEXT_SPECIFIC).is_some()
}
