/// Encapsulate IO actions

/// A message to use for scheduling IO actions
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Action {
    /// Input device should be read
    Read,

    /// Output device should be actuated
    On,

    /// Output device should be turned off
    Off,
}