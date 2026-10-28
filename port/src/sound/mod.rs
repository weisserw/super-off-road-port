//! Native source driver contracts; source/flag derivations live in audio.trace.md.
//! Immutable uploaded data addresses identify resources, never CPU execution or mutable RAM.
pub mod definitions;
pub mod host;
pub mod score;
pub mod sequencer;
pub mod synthesis;
pub mod voice;
