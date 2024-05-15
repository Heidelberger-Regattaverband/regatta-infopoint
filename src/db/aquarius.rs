use crate::{
    config::Config,
    db::{
        cache::{CacheTrait, Caches},
        model::{Club, Filters, Heat, Kiosk, Race, Regatta, Registration, Score, Statistics},
        tiberius::{TiberiusConnectionManager, TiberiusPool},
    },
};
use actix_identity::Identity;
use bb8::PooledConnection;
use futures::future::join3;
use log::debug;
use std::time::{Duration, Instant};

/// The type of the database connection.
pub type AquariusClient<'a> = PooledConnection<'a, TiberiusConnectionManager>;

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
    pub async fn new() -> Self {
        let active_regatta_id: i32 = if Config::get().active_regatta_id.is_none() {
            let start: Instant = Instant::now();
            let regatta = Regatta::query_active_regatta(TiberiusPool::instance()).await;
            debug!("Query active regatta from DB: {:?}", start.elapsed());
            regatta.id
        } else {
            Config::get().active_regatta_id.unwrap()
        };
        Aquarius {
            caches: Caches::new(Duration::from_secs(Config::get().cache_ttl)),
            active_regatta_id,
        }
    }

    pub async fn get_active_regatta(&self, opt_user: Option<Identity>) -> Regatta {
        self.get_regatta(self.active_regatta_id, opt_user).await
    }

    pub async fn get_filters(&self, regatta_id: i32, opt_user: Option<Identity>) -> Filters {
        if opt_user.is_some() {
            self._query_filters(regatta_id).await
        } else if let Some(filters) = self.caches.filters.get(&regatta_id).await {
            filters
        } else {
            self._query_filters(regatta_id).await
        }
    }

    pub async fn query_regattas(&self) -> Vec<Regatta> {
        let start = Instant::now();
        let regattas = Regatta::query_all(TiberiusPool::instance()).await;
        debug!("Query all regattas from DB: {:?}", start.elapsed());
        regattas
    }

    pub async fn get_regatta(&self, regatta_id: i32, opt_user: Option<Identity>) -> Regatta {
        if opt_user.is_some() {
            self._query_regatta(regatta_id).await
        } else if let Some(regatta) = self.caches.regatta.get(&regatta_id).await {
            regatta
        } else {
            self._query_regatta(regatta_id).await
        }
    }

    pub async fn get_races(&self, regatta_id: i32, opt_user: Option<Identity>) -> Vec<Race> {
        if opt_user.is_some() {
            self._query_races(regatta_id).await
        } else if let Some(races) = self.caches.races.get(&regatta_id).await {
            races
        } else {
            self._query_races(regatta_id).await
        }
    }

    pub(crate) async fn get_race_heats_registrations(&self, race_id: i32, opt_user: Option<Identity>) -> Race {
        if opt_user.is_some() {
            self._query_race_heats_registrations(race_id).await
        } else if let Some(race) = self.caches.race_heats_registrations.get(&race_id).await {
            race
        } else {
            self._query_race_heats_registrations(race_id).await
        }
    }

    pub async fn get_club(&self, club_id: i32) -> Club {
        if let Some(race) = self.caches.club.get(&club_id).await {
            race
        } else {
            self._query_club(club_id).await
        }
    }

    pub async fn get_heats(&self, regatta_id: i32, opt_user: Option<Identity>) -> Vec<Heat> {
        if opt_user.is_some() {
            self._query_heats(regatta_id).await
        } else if let Some(heats) = self.caches.heats.get(&regatta_id).await {
            heats
        } else {
            self._query_heats(regatta_id).await
        }
    }

    /// Returns heat details for the given identifier.
    pub async fn get_heat(&self, heat_id: i32, opt_user: Option<Identity>) -> Heat {
        if opt_user.is_some() {
            self._query_heat(heat_id).await
        } else if let Some(heat) = self.caches.heat.get(&heat_id).await {
            heat
        } else {
            self._query_heat(heat_id).await
        }
    }

    pub async fn get_participating_clubs(&self, regatta_id: i32, opt_user: Option<Identity>) -> Vec<Club> {
        if opt_user.is_some() {
            self._query_participating_clubs(regatta_id).await
        } else if let Some(clubs) = self.caches.participating_clubs.get(&regatta_id).await {
            clubs
        } else {
            self._query_participating_clubs(regatta_id).await
        }
    }

    pub(crate) async fn get_club_registrations(
        &self,
        regatta_id: i32,
        club_id: i32,
        opt_user: Option<Identity>,
    ) -> Vec<Registration> {
        if opt_user.is_some() {
            self._query_club_registrations(regatta_id, club_id).await
        } else if let Some(registrations) = self.caches.club_registrations.get(&(regatta_id, club_id)).await {
            registrations
        } else {
            self._query_club_registrations(regatta_id, club_id).await
        }
    }

    pub async fn calculate_scoring(&self, regatta_id: i32) -> Vec<Score> {
        let start = Instant::now();
        let scores = Score::calculate(regatta_id, TiberiusPool::instance()).await;
        debug!(
            "Calculate scoring of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        scores
    }

    pub async fn query_statistics(&self, regatta_id: i32) -> Statistics {
        let start = Instant::now();
        let stats = Statistics::query(regatta_id, TiberiusPool::instance()).await;
        debug!(
            "Query statistics of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        stats
    }

    pub async fn query_kiosk(&self, regatta_id: i32) -> Kiosk {
        let start = Instant::now();

        let finished = Kiosk::query_finished(regatta_id, &mut TiberiusPool::instance().get().await).await;
        let next = Kiosk::query_next(regatta_id, &mut TiberiusPool::instance().get().await).await;

        let kiosk = Kiosk {
            finished,
            next,
            running: Vec::with_capacity(0),
        };
        debug!("Query kiosk of regatta {} from DB: {:?}", regatta_id, start.elapsed());
        kiosk
    }

    async fn _query_filters(&self, regatta_id: i32) -> Filters {
        let start = Instant::now();
        let filters = Filters::query(regatta_id, TiberiusPool::instance()).await;
        self.caches.filters.set(&regatta_id, &filters).await;
        debug!("Query filters from DB: {:?}", start.elapsed());
        filters
    }

    async fn _query_regatta(&self, regatta_id: i32) -> Regatta {
        let start = Instant::now();
        let regatta = Regatta::query(regatta_id, TiberiusPool::instance()).await;
        self.caches.regatta.set(&regatta.id, &regatta).await;
        debug!("Query regatta {} from DB: {:?}", regatta_id, start.elapsed());
        regatta
    }

    async fn _query_race_heats_registrations(&self, race_id: i32) -> Race {
        let start = Instant::now();
        let result = join3(
            Race::query_race_by_id(race_id, TiberiusPool::instance()),
            Heat::query_heats_of_race(race_id, TiberiusPool::instance()),
            Registration::query_registrations_for_race(race_id, TiberiusPool::instance()),
        )
        .await;
        let mut race = result.0;
        if result.1.is_empty() {
            race.heats = None;
        } else {
            race.heats = Some(result.1);
        }
        if result.2.is_empty() {
            race.registrations = None;
        } else {
            race.registrations = Some(result.2);
        }
        self.caches.race_heats_registrations.set(&race.id, &race).await;
        debug!(
            "Query race {} with heats and registrations from DB: {:?}",
            race_id,
            start.elapsed()
        );
        race
    }

    async fn _query_races(&self, regatta_id: i32) -> Vec<Race> {
        let start = Instant::now();
        let races = Race::query_races_of_regatta(regatta_id, TiberiusPool::instance()).await;
        self.caches.races.set(&regatta_id, &races).await;
        debug!("Query races of regatta {} from DB: {:?}", regatta_id, start.elapsed());
        races
    }

    async fn _query_heats(&self, regatta_id: i32) -> Vec<Heat> {
        let start = Instant::now();
        let heats: Vec<Heat> = Heat::query_heats_of_regatta(regatta_id, TiberiusPool::instance()).await;
        self.caches.heats.set(&regatta_id, &heats).await;
        debug!("Query heats of regatta {} from DB: {:?}", regatta_id, start.elapsed());
        heats
    }

    async fn _query_heat(&self, heat_id: i32) -> Heat {
        let start = Instant::now();
        let heat = Heat::query_single(heat_id, TiberiusPool::instance()).await;
        self.caches.heat.set(&heat_id, &heat).await;
        debug!(
            "Query heat {} with registrations from DB: {:?}",
            heat_id,
            start.elapsed()
        );
        heat
    }

    async fn _query_participating_clubs(&self, regatta_id: i32) -> Vec<Club> {
        let start = Instant::now();
        let clubs = Club::query_clubs_participating_regatta(regatta_id, TiberiusPool::instance()).await;
        self.caches.participating_clubs.set(&regatta_id, &clubs).await;
        debug!(
            "Query participating clubs of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        clubs
    }

    async fn _query_club_registrations(&self, regatta_id: i32, club_id: i32) -> Vec<Registration> {
        let start = Instant::now();
        let registrations =
            Registration::query_registrations_of_club(regatta_id, club_id, TiberiusPool::instance()).await;
        self.caches
            .club_registrations
            .set(&(regatta_id, club_id), &registrations)
            .await;
        debug!(
            "Query registrations of club {} for regatta {} from DB: {:?}",
            club_id,
            regatta_id,
            start.elapsed()
        );
        registrations
    }

    async fn _query_club(&self, club_id: i32) -> Club {
        let start = Instant::now();
        let club = Club::query_club_by_id(club_id, TiberiusPool::instance()).await;
        self.caches.club.set(&club.id, &club).await;
        debug!("Query club {} from DB: {:?}", club_id, start.elapsed());
        club
    }
}
