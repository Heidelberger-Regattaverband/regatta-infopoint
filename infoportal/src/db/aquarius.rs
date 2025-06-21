use crate::{
    config::Config,
    db::cache::{CacheTrait, Caches},
};
use actix_identity::Identity;
use aquarius::{
    db::model::{Athlete, Club, Entry, Filters, Heat, Race, Regatta, Schedule, Score, Statistics},
    tiberius::TiberiusPool,
};
use futures::future::join3;
use log::debug;
use std::time::{Duration, Instant};
use tiberius::error::Error as DbError;

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
        let active_regatta_id: i32 = if Config::get().active_regatta_id.is_none() {
            let start = Instant::now();
            let regatta = Regatta::query_active_regatta(TiberiusPool::instance()).await?;
            debug!("Query active regatta from DB: {:?}", start.elapsed());
            regatta.id
        } else {
            Config::get().active_regatta_id.unwrap()
        };
        Ok(Aquarius {
            caches: Caches::new(Duration::from_secs(Config::get().cache_ttl)),
            active_regatta_id,
        })
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
        if opt_user.is_some() {
            self._query_filters(regatta_id).await
        } else if let Some(filters) = self.caches.filters.get(&regatta_id).await {
            Ok(filters)
        } else {
            self._query_filters(regatta_id).await
        }
    }

    async fn get_regatta(&self, regatta_id: i32, opt_user: Option<Identity>) -> Result<Option<Regatta>, DbError> {
        if opt_user.is_some() {
            self._query_regatta(regatta_id).await
        } else if let Some(regatta) = self.caches.regatta.get(&regatta_id).await {
            Ok(Some(regatta))
        } else {
            self._query_regatta(regatta_id).await
        }
    }

    pub(crate) async fn get_races(&self, regatta_id: i32, opt_user: Option<Identity>) -> Result<Vec<Race>, DbError> {
        if opt_user.is_some() {
            self._query_races(regatta_id).await
        } else if let Some(races) = self.caches.races.get(&regatta_id).await {
            Ok(races)
        } else {
            self._query_races(regatta_id).await
        }
    }

    pub(crate) async fn get_race_heats_entries(
        &self,
        race_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Race, DbError> {
        if opt_user.is_some() {
            self._query_race_heats_entries(race_id).await
        } else if let Some(race) = self.caches.race_heats_entries.get(&race_id).await {
            Ok(race)
        } else {
            self._query_race_heats_entries(race_id).await
        }
    }

    pub(crate) async fn get_regatta_club(
        &self,
        regatta_id: i32,
        club_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Club, DbError> {
        if opt_user.is_some() {
            self._query_club_with_aggregations(regatta_id, club_id).await
        } else if let Some(club) = self.caches.club_with_aggregations.get(&(regatta_id, club_id)).await {
            Ok(club)
        } else {
            self._query_club_with_aggregations(regatta_id, club_id).await
        }
    }

    async fn _query_club_with_aggregations(&self, regatta_id: i32, club_id: i32) -> Result<Club, DbError> {
        let start = Instant::now();
        let club = Club::query_club_with_aggregations(regatta_id, club_id, TiberiusPool::instance()).await?;
        self.caches
            .club_with_aggregations
            .set(&(regatta_id, club_id), &club)
            .await;
        debug!(
            "Query club {} for regatta {} from DB: {:?}",
            club_id,
            regatta_id,
            start.elapsed()
        );
        Ok(club)
    }

    pub(crate) async fn get_heats(&self, regatta_id: i32, opt_user: Option<Identity>) -> Result<Vec<Heat>, DbError> {
        if opt_user.is_some() {
            self._query_heats(regatta_id).await
        } else if let Some(heats) = self.caches.heats.get(&regatta_id).await {
            Ok(heats)
        } else {
            self._query_heats(regatta_id).await
        }
    }

    /// Returns heat details for the given identifier.
    pub(crate) async fn get_heat(&self, heat_id: i32, opt_user: Option<Identity>) -> Result<Heat, DbError> {
        if opt_user.is_some() {
            self._query_heat(heat_id).await
        } else if let Some(heat) = self.caches.heat.get(&heat_id).await {
            Ok(heat)
        } else {
            self._query_heat(heat_id).await
        }
    }

    pub(crate) async fn get_participating_clubs(
        &self,
        regatta_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Vec<Club>, DbError> {
        if opt_user.is_some() {
            self._query_participating_clubs(regatta_id).await
        } else if let Some(clubs) = self.caches.clubs.get(&regatta_id).await {
            Ok(clubs)
        } else {
            self._query_participating_clubs(regatta_id).await
        }
    }

    pub(crate) async fn get_club_entries(
        &self,
        regatta_id: i32,
        club_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Vec<Entry>, DbError> {
        if opt_user.is_some() {
            self._query_club_entries(regatta_id, club_id).await
        } else if let Some(entries) = self.caches.club_entries.get(&(regatta_id, club_id)).await {
            Ok(entries)
        } else {
            self._query_club_entries(regatta_id, club_id).await
        }
    }

    pub(crate) async fn get_participating_athletes(
        &self,
        regatta_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Vec<Athlete>, DbError> {
        if opt_user.is_some() {
            self._query_athletes(regatta_id).await
        } else if let Some(entries) = self.caches.athletes.get(&regatta_id).await {
            Ok(entries)
        } else {
            self._query_athletes(regatta_id).await
        }
    }

    pub(crate) async fn get_athlete_entries(
        &self,
        regatta_id: i32,
        athlete_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Vec<Entry>, DbError> {
        if opt_user.is_some() {
            self._query_athlete_entries(regatta_id, athlete_id).await
        } else if let Some(entries) = self.caches.athlete_entries.get(&(regatta_id, athlete_id)).await {
            Ok(entries)
        } else {
            self._query_athlete_entries(regatta_id, athlete_id).await
        }
    }

    pub(crate) async fn get_athlete(
        &self,
        regatta_id: i32,
        athlete_id: i32,
        opt_user: Option<Identity>,
    ) -> Result<Athlete, DbError> {
        if opt_user.is_some() {
            self._query_athlete(regatta_id, athlete_id).await
        } else if let Some(athlete) = self.caches.athlete.get(&athlete_id).await {
            Ok(athlete)
        } else {
            self._query_athlete(regatta_id, athlete_id).await
        }
    }

    pub(crate) async fn calculate_scoring(&self, regatta_id: i32) -> Result<Vec<Score>, DbError> {
        let start = Instant::now();
        let scores = Score::calculate(regatta_id, TiberiusPool::instance()).await;
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
        if opt_user.is_some() {
            self._query_schedule(regatta_id).await
        } else if let Some(entries) = self.caches.schedule.get(&(regatta_id)).await {
            Ok(entries)
        } else {
            self._query_schedule(regatta_id).await
        }
    }

    // private methods for querying the database

    async fn _query_filters(&self, regatta_id: i32) -> Result<Filters, DbError> {
        let start = Instant::now();
        let filters = Filters::query(regatta_id, TiberiusPool::instance()).await?;
        self.caches.filters.set(&regatta_id, &filters).await;
        debug!("Query filters from DB: {:?}", start.elapsed());
        Ok(filters)
    }

    async fn _query_regatta(&self, regatta_id: i32) -> Result<Option<Regatta>, DbError> {
        let start = Instant::now();
        let regatta = Regatta::query_by_id(regatta_id, TiberiusPool::instance()).await?;
        if let Some(regatta) = &regatta {
            self.caches.regatta.set(&regatta_id, regatta).await;
        }
        debug!("Query regatta {} from DB: {:?}", regatta_id, start.elapsed());
        Ok(regatta)
    }

    async fn _query_race_heats_entries(&self, race_id: i32) -> Result<Race, DbError> {
        let start = Instant::now();
        let queries = join3(
            Race::query_race_by_id(race_id, TiberiusPool::instance()),
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
        self.caches.race_heats_entries.set(&race.id, &race).await;
        debug!(
            "Query race {} with heats and entries from DB: {:?}",
            race_id,
            start.elapsed()
        );
        Ok(race)
    }

    async fn _query_races(&self, regatta_id: i32) -> Result<Vec<Race>, DbError> {
        let start = Instant::now();
        let races = Race::query_races_of_regatta(regatta_id, TiberiusPool::instance()).await?;
        self.caches.races.set(&regatta_id, &races).await;
        debug!("Query races of regatta {} from DB: {:?}", regatta_id, start.elapsed());
        Ok(races)
    }

    async fn _query_heats(&self, regatta_id: i32) -> Result<Vec<Heat>, DbError> {
        let start = Instant::now();
        let heats: Vec<Heat> = Heat::query_heats_of_regatta(regatta_id, TiberiusPool::instance()).await?;
        self.caches.heats.set(&regatta_id, &heats).await;
        debug!("Query heats of regatta {} from DB: {:?}", regatta_id, start.elapsed());
        Ok(heats)
    }

    async fn _query_heat(&self, heat_id: i32) -> Result<Heat, DbError> {
        let start = Instant::now();
        let heat = Heat::query_single(heat_id, TiberiusPool::instance()).await?;
        self.caches.heat.set(&heat_id, &heat).await;
        debug!("Query heat {} with entries from DB: {:?}", heat_id, start.elapsed());
        Ok(heat)
    }

    async fn _query_participating_clubs(&self, regatta_id: i32) -> Result<Vec<Club>, DbError> {
        let start = Instant::now();
        let clubs = Club::query_clubs_participating_regatta(regatta_id, TiberiusPool::instance()).await?;
        self.caches.clubs.set(&regatta_id, &clubs).await;
        debug!(
            "Query participating clubs of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        Ok(clubs)
    }

    async fn _query_club_entries(&self, regatta_id: i32, club_id: i32) -> Result<Vec<Entry>, DbError> {
        let start = Instant::now();
        let entries = Entry::query_entries_of_club(regatta_id, club_id, TiberiusPool::instance()).await?;
        self.caches.club_entries.set(&(regatta_id, club_id), &entries).await;
        debug!(
            "Query entries of club {} for regatta {} from DB: {:?}",
            club_id,
            regatta_id,
            start.elapsed()
        );
        Ok(entries)
    }

    async fn _query_athletes(&self, regatta_id: i32) -> Result<Vec<Athlete>, DbError> {
        let start = Instant::now();
        let athletes = Athlete::query_participating_athletes(regatta_id, TiberiusPool::instance()).await?;
        self.caches.athletes.set(&regatta_id, &athletes).await;
        debug!(
            "Query athletes of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        Ok(athletes)
    }

    async fn _query_athlete_entries(&self, regatta_id: i32, athlete_id: i32) -> Result<Vec<Entry>, DbError> {
        let start = Instant::now();
        let entries = Entry::query_entries_of_athlete(regatta_id, athlete_id, TiberiusPool::instance()).await?;
        self.caches
            .athlete_entries
            .set(&(regatta_id, athlete_id), &entries)
            .await;
        debug!(
            "Query entries of athlete {} for regatta {} from DB: {:?}",
            athlete_id,
            regatta_id,
            start.elapsed()
        );
        Ok(entries)
    }

    async fn _query_athlete(&self, regatta_id: i32, athlete_id: i32) -> Result<Athlete, DbError> {
        let start = Instant::now();
        let athlete = Athlete::query_athlete(regatta_id, athlete_id, TiberiusPool::instance()).await?;
        self.caches.athlete.set(&athlete_id, &athlete).await;
        debug!("Query athlete {} from DB: {:?}", athlete_id, start.elapsed());
        Ok(athlete)
    }

    async fn _query_schedule(&self, regatta_id: i32) -> Result<Schedule, DbError> {
        let start = Instant::now();
        let schedule = Schedule::query_schedule_for_regatta(regatta_id, TiberiusPool::instance()).await?;
        self.caches.schedule.set(&regatta_id, &schedule).await;
        debug!(
            "Query schedule of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        Ok(schedule)
    }
}
