use crate::config::CONFIG;
use ::db::aquarius::model::Message;
use actix_identity::Identity;
use db::{
    aquarius::model::{Athlete, Club, Entry, Filters, Heat, Race, Regatta, Schedule, Score, Statistics},
    cache::{CacheStats, Caches},
    error::DbError,
    tiberius::TiberiusPool,
};
use futures::future::join3;
use std::time::{Duration, Instant};
use tracing::debug;

/// The `Aquarius` struct is the main interface to the database. It is used to query data from the database.
pub(crate) struct Aquarius {
    /// The caches for the database queries.
    caches: Caches,

    /// The identifier of the active regatta.
    active_regatta_id: i32,
}

/// Implementation of the `Aquarius` struct.
impl Aquarius {
    /// Creates a new `Aquarius`.
    pub(crate) async fn new() -> Result<Self, DbError> {
        let active_regatta_id: i32 = match CONFIG.active_regatta_id {
            Some(id) => id,
            None => {
                Regatta::query_active_regatta(&mut *TiberiusPool::instance().get().await?)
                    .await?
                    .id
            }
        };
        Ok(Aquarius {
            caches: Caches::try_new(Duration::from_secs(CONFIG.cache_ttl))?,
            active_regatta_id,
        })
    }

    pub(crate) fn get_cache_stats(&self) -> CacheStats {
        self.caches.get_summary_stats()
    }

    /// Returns the active regatta.
    /// # Arguments
    /// * `opt_user` - The optional user identity.
    /// # Returns
    /// The active regatta.
    /// # Errors
    /// Returns a `DbError` if the query fails.
    pub(crate) async fn get_active_regatta(&self, opt_user: Option<Identity>) -> Result<Option<Regatta>, DbError> {
        self.get_regatta(self.active_regatta_id, opt_user).await
    }

    /// Returns filters for the given regatta.
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    /// * `opt_user` - The optional user identity.
    /// # Returns
    /// The filters for the given regatta.  
    /// # Errors
    /// Returns a `DbError` if the query fails.
    pub(crate) async fn get_filters(&self, regatta_id: i32, opt_user: Option<Identity>) -> Result<Filters, DbError> {
        self.caches
            .filters
            .compute_if_missing(&regatta_id, opt_user.is_some(), || async move {
                let start = Instant::now();
                let filters = Filters::query(regatta_id, TiberiusPool::instance()).await?;
                debug!("Query filters from DB: {:?}", start.elapsed());
                Ok::<Filters, DbError>(filters)
            })
            .await
    }

    async fn get_regatta(&self, regatta_id: i32, opt_user: Option<Identity>) -> Result<Option<Regatta>, DbError> {
        self.caches
            .regattas
            .compute_if_missing_opt(&regatta_id, opt_user.is_some(), || async move {
                let start = Instant::now();
                let regatta = Regatta::query_by_id(regatta_id, TiberiusPool::instance()).await?;
                debug!("Query regatta {} from DB: {:?}", regatta_id, start.elapsed());
                Ok::<Option<Regatta>, DbError>(regatta)
            })
            .await
    }

    pub(crate) async fn get_races(&self, regatta_id: i32, opt_user: Option<Identity>) -> Result<Vec<Race>, DbError> {
        self.caches
            .races
            .compute_if_missing(&regatta_id, opt_user.is_some(), || async move {
                let start = Instant::now();
                let races =
                    Race::query_races_of_regatta(regatta_id, &mut *TiberiusPool::instance().get().await?).await?;
                debug!("Query races of regatta {} from DB: {:?}", regatta_id, start.elapsed());
                Ok::<Vec<Race>, DbError>(races)
            })
            .await
    }

    pub(crate) async fn get_race_heats_entries(
        &self,
        race_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Race, DbError> {
        self.caches
            .race_heats_entries
            .compute_if_missing(&race_id, opt_user.is_some(), || self.query_race_heats_entries(race_id))
            .await
    }

    pub(crate) async fn get_regatta_club(
        &self,
        regatta_id: i32,
        club_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Club, DbError> {
        self.caches
            .club_with_aggregations
            .compute_if_missing(&(regatta_id, club_id), opt_user.is_some(), || async move {
                let start = Instant::now();
                let club = Club::query_club_with_aggregations(regatta_id, club_id, TiberiusPool::instance()).await?;
                debug!(
                    "Query club {} for regatta {} from DB: {:?}",
                    club_id,
                    regatta_id,
                    start.elapsed()
                );
                Ok::<Club, DbError>(club)
            })
            .await
    }

    pub(crate) async fn get_heats(&self, regatta_id: i32, opt_user: Option<Identity>) -> Result<Vec<Heat>, DbError> {
        self.caches
            .heats
            .compute_if_missing(&regatta_id, opt_user.is_some(), || async move {
                let start = Instant::now();
                let heats: Vec<Heat> = Heat::query_heats_of_regatta(regatta_id, TiberiusPool::instance()).await?;
                debug!("Query heats of regatta {} from DB: {:?}", regatta_id, start.elapsed());
                Ok::<Vec<Heat>, DbError>(heats)
            })
            .await
    }

    /// Returns heat details for the given identifier.
    pub(crate) async fn get_heat(&self, heat_id: i32, opt_user: Option<Identity>) -> Result<Heat, DbError> {
        self.caches
            .heat
            .compute_if_missing(&heat_id, opt_user.is_some(), || async move {
                let start = Instant::now();
                let heat = Heat::query_single(heat_id, TiberiusPool::instance()).await?;
                debug!("Query heat {} with entries from DB: {:?}", heat_id, start.elapsed());
                Ok::<Heat, DbError>(heat)
            })
            .await
    }

    pub(crate) async fn get_participating_clubs(
        &self,
        regatta_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Vec<Club>, DbError> {
        self.caches
            .clubs
            .compute_if_missing(&regatta_id, opt_user.is_some(), || async move {
                let start = Instant::now();
                let clubs = Club::query_clubs_participating_regatta(regatta_id, TiberiusPool::instance()).await?;
                debug!(
                    "Query participating clubs of regatta {} from DB: {:?}",
                    regatta_id,
                    start.elapsed()
                );
                Ok::<Vec<Club>, DbError>(clubs)
            })
            .await
    }

    pub(crate) async fn get_club_entries(
        &self,
        regatta_id: i32,
        club_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Vec<Entry>, DbError> {
        self.caches
            .club_entries
            .compute_if_missing(&(regatta_id, club_id), opt_user.is_some(), || async move {
                let start = Instant::now();
                let entries = Entry::query_entries_of_club(regatta_id, club_id, TiberiusPool::instance()).await?;
                debug!(
                    "Query entries of club {} for regatta {} from DB: {:?}",
                    club_id,
                    regatta_id,
                    start.elapsed()
                );
                Ok::<Vec<Entry>, DbError>(entries)
            })
            .await
    }

    pub(crate) async fn get_participating_athletes(
        &self,
        regatta_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Vec<Athlete>, DbError> {
        self.caches
            .athletes
            .compute_if_missing(&regatta_id, opt_user.is_some(), || async move {
                let start = Instant::now();
                let athletes =
                    Athlete::query_participating_athletes(regatta_id, &mut *TiberiusPool::instance().get().await?)
                        .await?;
                debug!(
                    "Query athletes of regatta {} from DB: {:?}",
                    regatta_id,
                    start.elapsed()
                );
                Ok::<Vec<Athlete>, DbError>(athletes)
            })
            .await
    }

    pub(crate) async fn get_athlete_entries(
        &self,
        regatta_id: i32,
        athlete_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Vec<Entry>, DbError> {
        self.caches
            .athlete_entries
            .compute_if_missing(&(regatta_id, athlete_id), opt_user.is_some(), || async move {
                let start = Instant::now();
                let entries = Entry::query_entries_of_athlete(regatta_id, athlete_id, TiberiusPool::instance()).await?;
                debug!(
                    "Query entries of athlete {} for regatta {} from DB: {:?}",
                    athlete_id,
                    regatta_id,
                    start.elapsed()
                );
                Ok::<Vec<Entry>, DbError>(entries)
            })
            .await
    }

    pub(crate) async fn get_athlete(
        &self,
        regatta_id: i32,
        athlete_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Athlete, DbError> {
        self.caches
            .athlete
            .compute_if_missing(&athlete_id, opt_user.is_some(), || async move {
                let start = Instant::now();
                let athlete =
                    Athlete::query_athlete(regatta_id, athlete_id, &mut *TiberiusPool::instance().get().await?).await?;
                debug!("Query athlete {} from DB: {:?}", athlete_id, start.elapsed());
                Ok::<Athlete, DbError>(athlete)
            })
            .await
    }

    pub(crate) async fn calculate_scoring(&self, regatta_id: i32) -> Result<Vec<Score>, DbError> {
        let start = Instant::now();
        let scores = Score::calculate(regatta_id, &mut *TiberiusPool::instance().get().await?).await;
        debug!(
            "Calculate scoring of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        scores
    }

    pub(crate) async fn query_statistics(&self, regatta_id: i32) -> Result<Statistics, DbError> {
        let start = Instant::now();
        let stats = Statistics::query(regatta_id, TiberiusPool::instance()).await;
        debug!(
            "Query statistics of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        stats
    }

    pub(crate) async fn query_schedule(
        &self,
        regatta_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Schedule, DbError> {
        self.caches
            .schedule
            .compute_if_missing(&regatta_id, opt_user.is_some(), || async move {
                let start = Instant::now();
                let schedule =
                    Schedule::query_schedule_for_regatta(regatta_id, &mut *TiberiusPool::instance().get().await?)
                        .await?;
                debug!(
                    "Query schedule of regatta {} from DB: {:?}",
                    regatta_id,
                    start.elapsed()
                );
                Ok::<Schedule, DbError>(schedule)
            })
            .await
    }

    pub(crate) async fn get_messages(&self, regatta_id: i32) -> Result<Vec<Message>, DbError> {
        let start = Instant::now();
        let messages =
            Message::query_messages_for_regatta(regatta_id, &mut *TiberiusPool::instance().get().await?).await?;
        debug!(
            "Query messages of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        Ok(messages)
    }

    // private methods for querying the database

    async fn query_race_heats_entries(&self, race_id: i32) -> Result<Race, DbError> {
        let start = Instant::now();
        let queries = join3(
            Race::query_race_by_id(race_id, &mut *TiberiusPool::instance().get().await?),
            Heat::query_heats_of_race(race_id, TiberiusPool::instance()),
            Entry::query_entries_for_race(race_id, TiberiusPool::instance()),
        )
        .await;
        let mut race = queries.0?;
        let heats = queries.1?;
        if heats.is_empty() {
            race.heats = None;
        } else {
            race.heats = Some(heats);
        }
        let entries = queries.2?;
        if entries.is_empty() {
            race.entries = None;
        } else {
            race.entries = Some(entries);
        }
        debug!(
            "Query race {} with heats and entries from DB: {:?}",
            race_id,
            start.elapsed()
        );
        Ok(race)
    }
}
