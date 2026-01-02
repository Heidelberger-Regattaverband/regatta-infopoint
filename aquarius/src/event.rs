use crate::messages::EventHeatChanged;

/// Events emitted by the Aquarius client.
#[derive(Debug)]
pub enum AquariusEvent {
    /// An event indicating that the list of heats has changed
    HeatListChanged(EventHeatChanged),

    /// An event from the client, e.g. connection lost
    Client(bool),
}
