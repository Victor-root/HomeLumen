/// Result of any driver operation.
pub type Result<T> = std::result::Result<T, Error>;

/// What can go wrong while talking to a device.
///
/// The variants are deliberately coarse: the engine only needs to know whether
/// a route is still worth trying.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The device did not answer in time, or the network refused the route.
    #[error("appareil injoignable: {0}")]
    Unreachable(String),

    /// The device answered, but rejected the request.
    #[error("l'appareil a refusé la commande: {0}")]
    Rejected(String),

    /// The answer could not be understood.
    #[error("réponse illisible: {0}")]
    Protocol(String),

    /// The device cannot do what was asked of it.
    #[error("fonction non supportée: {0}")]
    Unsupported(String),

    /// Credentials are missing, expired or wrong.
    #[error("authentification refusée: {0}")]
    Unauthorized(String),
}
