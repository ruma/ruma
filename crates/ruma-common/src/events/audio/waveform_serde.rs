//! `Serialize` and `Deserialize` implementations for extensible events (MSC1767).

use serde::Deserialize;

use super::{Amplitude, Waveform, WaveformError};

#[derive(Debug, Default, Deserialize)]
pub(crate) struct WaveformSerDeHelper(Vec<Amplitude>);

impl TryFrom<WaveformSerDeHelper> for Waveform {
    type Error = WaveformError;

    fn try_from(helper: WaveformSerDeHelper) -> Result<Self, Self::Error> {
        Waveform::try_from(helper.0)
    }
}
