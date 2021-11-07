/// Module: An overlay (with N number of I/O pins) that sits "on" an integrated circuit. All modules
/// belong to exactly one chunk, and are keyed on their "root" location. This location does not need
/// to be coincident a pin however, nor does a module need to have any pins at all (ex. a label).
/// Pins are not stored themselves, but are provided by the specific module. However, the UPC format
/// encodes pin presence, and that must always line up one-for-one with the module provided pins;
/// pin generation must be deterministic and idempotent.
#[derive(Clone)]
pub struct Module {}
