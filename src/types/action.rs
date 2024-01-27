/// Encapsulate IO actions
///
/// IO actions are used for both input and output devices. For example, a temperature sensor may
/// be polled for its current temperature, and a heater may be turned on or off.
///
/// These actions are used to schedule IO events and is used within [`crate::controllers::Controller`]s
/// to keep track of future events.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Action {
    /// Input device should be read
    Read,

    /// Output device should be activated
    On,

    /// Output device should be deactivated
    Off,
}