use ::aquarius::error::AquariusErr;
use ::db::error::DbError;
use ::db::tiberius_client::error::Error as TiberiusError;
use ::thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum TimekeeperErr {
    #[error("Tiberius error: {0}")]
    Tiberius(#[from] TiberiusError),
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    #[error("Aquarius error: {0}")]
    Aquarius(#[from] AquariusErr),
}
