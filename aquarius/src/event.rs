use crate::messages::EventHeatChanged;

/// Events emitted by the Aquarius client.
pub enum AquariusEvent {
    /// An event from Aquarius indicating that a heat has changed.
    HeatChanged(EventHeatChanged),

    /// An event from the client, e.g. connection lost
    Client(bool),
}
