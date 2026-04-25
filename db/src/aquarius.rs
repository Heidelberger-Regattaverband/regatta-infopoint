mod flags_scraper;
pub mod model;

use crate::aquarius::model::Athlete;
use crate::aquarius::model::Club;
use crate::aquarius::model::CreateNotificationRequest;
use crate::aquarius::model::Entry;
use crate::aquarius::model::Filters;
use crate::aquarius::model::Heat;
use crate::aquarius::model::Notification;
use crate::aquarius::model::Race;
use crate::aquarius::model::Regatta;
use crate::aquarius::model::Schedule;
use crate::aquarius::model::Score;
use crate::aquarius::model::Statistics;
use crate::aquarius::model::UpdateNotificationRequest;
use crate::cache::CacheStats;
use crate::cache::Caches;
use crate::error::DbError;
use crate::tiberius::TiberiusPool;
use ::futures::future::join3;
use ::std::time::{Duration, Instant};
use ::tracing::debug;

/// Executes an async expression, measures its elapsed time, and logs it via `debug!`.
///
/// Usage:
/// ```ignore
/// timed_query!("label", async_expr)
/// timed_query!("label", async_expr, key1 = val1, key2)
/// ```
macro_rules! timed_query {
    ($label:expr, $query:expr $(, $($key:ident $(= $val:expr)?),+)?) => {{
        let start = Instant::now();
        let result = $query;
        debug!($($($key $(= $val)?,)+)? elapsed = ?start.elapsed(), $label);
        result
    }};
}

/// The `Aquarius` struct is the main interface to the database. It is used to query data from the database.
pub struct Aquarius {
    /// The caches for the database queries.
    caches: Caches,

    /// The identifier of the active regatta.
    active_regatta_id: i32,
}

/// Implementation of the `Aquarius` struct.
impl Aquarius {
    /// Creates a new `Aquarius`.
    pub async fn new(active_regatta_id: Option<i32>, cache_ttl: u64) -> Result<Self, DbError> {
        let active_regatta_id: i32 = match active_regatta_id {
            Some(id) => id,
            None => {
                Regatta::query_active_regatta(&mut *TiberiusPool::instance().get().await?)
                    .await?
                    .id
            }
        };

        Ok(Aquarius {
            caches: Caches::try_new(Duration::from_secs(cache_ttl))?,
            active_regatta_id,
        })
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        self.caches.get_summary_stats()
    }

    /// Returns the active regatta.
    pub async fn get_active_regatta(&self, force_cache: bool) -> Result<Option<Regatta>, DbError> {
        self.get_regatta(self.active_regatta_id, force_cache).await
    }

    /// Returns filters for the given regatta.
    pub async fn get_filters(&self, regatta_id: i32, force_cache: bool) -> Result<Filters, DbError> {
        self.caches
            .filters
            .compute_if_missing(&regatta_id, force_cache, || async move {
                timed_query!(
                    "Query filters from DB:",
                    Filters::query(regatta_id, TiberiusPool::instance()).await
                )
            })
            .await
    }

    async fn get_regatta(&self, regatta_id: i32, force_cache: bool) -> Result<Option<Regatta>, DbError> {
        self.caches
            .regattas
            .compute_if_missing_opt(&regatta_id, force_cache, || async move {
                timed_query!(
                    "Query regatta from DB:",
                    Regatta::query_by_id(regatta_id, &mut *TiberiusPool::instance().get().await?).await,
                    regatta_id
                )
            })
            .await
    }

    pub async fn get_races(&self, regatta_id: i32, force_cache: bool) -> Result<Vec<Race>, DbError> {
        self.caches
            .races
            .compute_if_missing(&regatta_id, force_cache, || async move {
                timed_query!(
                    "Query races from DB:",
                    Race::query_races_of_regatta(regatta_id, &mut *TiberiusPool::instance().get().await?).await,
                    regatta_id
                )
            })
            .await
    }

    pub async fn get_race_heats_entries(&self, race_id: i32, force_cache: bool) -> Result<Race, DbError> {
        self.caches
            .race_heats_entries
            .compute_if_missing(&race_id, force_cache, || self.query_race_heats_entries(race_id))
            .await
    }

    pub async fn get_regatta_club(&self, regatta_id: i32, club_id: i32, force_cache: bool) -> Result<Club, DbError> {
        self.caches
            .club_with_aggregations
            .compute_if_missing(&(regatta_id, club_id), force_cache, || async move {
                timed_query!(
                    "Query club from DB:",
                    Club::query_club_with_aggregations(
                        regatta_id,
                        club_id,
                        &mut *TiberiusPool::instance().get().await?
                    )
                    .await,
                    regatta_id,
                    club_id
                )
            })
            .await
    }

    pub async fn get_heats(&self, regatta_id: i32, force_cache: bool) -> Result<Vec<Heat>, DbError> {
        self.caches
            .heats
            .compute_if_missing(&regatta_id, force_cache, || async move {
                timed_query!(
                    "Query heats from DB:",
                    Heat::query_heats_of_regatta(regatta_id, TiberiusPool::instance()).await,
                    regatta_id
                )
            })
            .await
    }

    pub async fn get_heats_ready_to_start(&self) -> Result<Vec<Heat>, DbError> {
        timed_query!(
            "Query heats ready to start from DB:",
            Heat::query_heats_ready_to_start(self.active_regatta_id, TiberiusPool::instance()).await
        )
    }

    pub async fn get_heats_started(&self) -> Result<Vec<Heat>, DbError> {
        timed_query!(
            "Query heats started from DB:",
            Heat::query_heats_started(self.active_regatta_id, TiberiusPool::instance()).await
        )
    }

    /// Returns heat details for the given identifier.
    pub async fn get_heat(&self, heat_id: i32, force_cache: bool) -> Result<Heat, DbError> {
        self.caches
            .heat
            .compute_if_missing(&heat_id, force_cache, || async move {
                timed_query!(
                    "Query heat with entries from DB:",
                    Heat::query_single(heat_id, TiberiusPool::instance()).await,
                    heat_id
                )
            })
            .await
    }

    pub async fn get_participating_clubs(&self, regatta_id: i32, force_cache: bool) -> Result<Vec<Club>, DbError> {
        self.caches
            .clubs
            .compute_if_missing(&regatta_id, force_cache, || async move {
                timed_query!(
                    "Query participating clubs from DB:",
                    Club::query_clubs_participating_regatta(regatta_id, &mut *TiberiusPool::instance().get().await?)
                        .await,
                    regatta_id
                )
            })
            .await
    }

    pub async fn get_club_entries(
        &self,
        regatta_id: i32,
        club_id: i32,
        force_cache: bool,
    ) -> Result<Vec<Entry>, DbError> {
        self.caches
            .club_entries
            .compute_if_missing(&(regatta_id, club_id), force_cache, || async move {
                timed_query!(
                    "Query entries of club from DB:",
                    Entry::query_entries_of_club(regatta_id, club_id, TiberiusPool::instance()).await,
                    regatta_id,
                    club_id
                )
            })
            .await
    }

    pub async fn get_participating_athletes(
        &self,
        regatta_id: i32,
        force_cache: bool,
    ) -> Result<Vec<Athlete>, DbError> {
        self.caches
            .athletes
            .compute_if_missing(&regatta_id, force_cache, || async move {
                timed_query!(
                    "Query athletes from DB:",
                    Athlete::query_participating_athletes(regatta_id, &mut *TiberiusPool::instance().get().await?)
                        .await,
                    regatta_id
                )
            })
            .await
    }

    pub async fn get_athlete_entries(
        &self,
        regatta_id: i32,
        athlete_id: i32,
        force_cache: bool,
    ) -> Result<Vec<Entry>, DbError> {
        self.caches
            .athlete_entries
            .compute_if_missing(&(regatta_id, athlete_id), force_cache, || async move {
                timed_query!(
                    "Query entries of athlete from DB:",
                    Entry::query_entries_of_athlete(regatta_id, athlete_id, TiberiusPool::instance()).await,
                    athlete_id,
                    regatta_id
                )
            })
            .await
    }

    pub async fn get_athlete(&self, regatta_id: i32, athlete_id: i32, force_cache: bool) -> Result<Athlete, DbError> {
        self.caches
            .athlete
            .compute_if_missing(&(regatta_id, athlete_id), force_cache, || async move {
                timed_query!(
                    "Query athlete from DB:",
                    Athlete::query_athlete(regatta_id, athlete_id, &mut *TiberiusPool::instance().get().await?).await,
                    regatta_id,
                    athlete_id
                )
            })
            .await
    }

    pub async fn calculate_scoring(&self, regatta_id: i32) -> Result<Vec<Score>, DbError> {
        timed_query!(
            "Calculate scoring from DB:",
            Score::calculate(regatta_id, &mut *TiberiusPool::instance().get().await?).await,
            regatta_id
        )
    }

    pub async fn query_statistics(&self, regatta_id: i32) -> Result<Statistics, DbError> {
        timed_query!(
            "Query statistics from DB:",
            Statistics::query(regatta_id, TiberiusPool::instance()).await,
            regatta_id
        )
    }

    pub async fn query_schedule(&self, regatta_id: i32, force_cache: bool) -> Result<Schedule, DbError> {
        self.caches
            .schedule
            .compute_if_missing(&regatta_id, force_cache, || async move {
                timed_query!(
                    "Query schedule from DB:",
                    Schedule::query_schedule_for_regatta(regatta_id, &mut *TiberiusPool::instance().get().await?).await,
                    regatta_id
                )
            })
            .await
    }

    pub async fn get_visible_notifications(&self, regatta_id: i32) -> Result<Vec<Notification>, DbError> {
        self.caches
            .notifications
            .compute_if_missing(&regatta_id, false, || async move {
                timed_query!(
                    "Query notifications from DB:",
                    Notification::query_visible_notifications_for_regatta(
                        regatta_id,
                        &mut *TiberiusPool::instance().get().await?
                    )
                    .await,
                    regatta_id
                )
            })
            .await
    }

    pub async fn get_all_notifications(
        &self,
        regatta_id: i32,
        user_pool: &TiberiusPool,
    ) -> Result<Vec<Notification>, DbError> {
        timed_query!(
            "Query all notifications from DB:",
            {
                let notifications =
                    Notification::query_all_notifications_for_regatta(regatta_id, &mut *user_pool.get().await?).await?;
                Ok(notifications)
            },
            regatta_id
        )
    }

    pub async fn create_notification(
        &self,
        regatta_id: i32,
        request: &CreateNotificationRequest,
        user_pool: &TiberiusPool,
    ) -> Result<Notification, DbError> {
        let start = Instant::now();
        let notification = Notification::create_notification(regatta_id, request, &mut *user_pool.get().await?).await?;
        self.caches.notifications.invalidate(&regatta_id).await?;
        debug!(regatta_id, elapsed = ?start.elapsed(), "Create notification in DB:");
        Ok(notification)
    }

    pub async fn update_notification(
        &self,
        notification_id: i32,
        request: &UpdateNotificationRequest,
        user_pool: &TiberiusPool,
    ) -> Result<Option<Notification>, DbError> {
        let start = Instant::now();
        let notification =
            Notification::update_notification(notification_id, request, &mut *user_pool.get().await?).await?;
        if let Some(notification) = &notification {
            self.caches.notifications.invalidate(&notification.event_id).await?;
        }
        debug!(notification_id, elapsed = ?start.elapsed(), "Update notification in DB:");
        Ok(notification)
    }

    pub async fn delete_notification(&self, notification_id: i32, user_pool: &TiberiusPool) -> Result<bool, DbError> {
        let start = Instant::now();
        let deleted = Notification::delete_notification(notification_id, &mut *user_pool.get().await?).await?;
        if let Some(notification) = &deleted {
            self.caches.notifications.invalidate(&notification.event_id).await?;
        }
        debug!(notification_id, elapsed = ?start.elapsed(), "Delete notification in DB:");
        Ok(deleted.is_some())
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
        debug!(race_id, elapsed = ?start.elapsed(), "Query race with heats and entries from DB:");
        Ok(race)
    }
}
