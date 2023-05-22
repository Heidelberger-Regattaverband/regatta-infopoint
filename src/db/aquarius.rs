use super::{
    cache::{CacheTrait, Caches},
    model::{Club, Crew, Heat, HeatRegistration, Kiosk, Race, Regatta, Registration, Score, Statistics},
    tiberius::{TiberiusConnectionManager, TiberiusPool},
};
use actix_identity::Identity;
use bb8::PooledConnection;
use colored::Colorize;
use log::{debug, info};
use std::{
    env,
    time::{Duration, Instant},
};
use tiberius::{Query, Row};

pub type AquariusClient<'a> = PooledConnection<'a, TiberiusConnectionManager>;

pub struct Aquarius {
    caches: Caches,
    pub pool: TiberiusPool,
    active_regatta_id: i32,
}

impl Aquarius {
    /// Create a new `Aquarius`.
    pub async fn new() -> Self {
        let active_regatta_id: i32 = env::var("ACTIVE_REGATTA_ID")
            .expect("env variable `ACTIVE_REGATTA_ID` should be set")
            .parse()
            .unwrap();
        let cache_ttl: u64 = env::var("CACHE_TTL")
            .expect("env variable `CACHE_TTL` should be set")
            .parse()
            .unwrap();
        info!(
            "Aquarius: active_regatta_id={}, cache_ttl={}s",
            active_regatta_id.to_string().bold(),
            cache_ttl.to_string().bold()
        );

        Aquarius {
            caches: Caches::new(Duration::from_secs(cache_ttl)),
            pool: TiberiusPool::new().await,
            active_regatta_id,
        }
    }

    pub async fn get_active_regatta(&self) -> Regatta {
        self.get_regatta(self.active_regatta_id).await
    }

    pub async fn query_regattas(&self) -> Vec<Regatta> {
        let start = Instant::now();
        let regattas = Regatta::query_all(&mut self.pool.get().await).await;
        debug!("Query all regattas from DB: {:?}", start.elapsed());
        regattas
    }

    pub async fn get_regatta(&self, regatta_id: i32) -> Regatta {
        if let Some(regatta) = self.caches.regatta.get(&regatta_id).await {
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

    pub async fn get_statistics(&self, regatta_id: i32) -> Statistics {
        let start = Instant::now();
        let stats = Statistics::query(regatta_id, &mut self.pool.get().await).await;
        debug!(
            "Query statistics of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        stats
    }

    pub async fn get_race(&self, race_id: i32) -> Race {
        if let Some(race) = self.caches.race.get(&race_id).await {
            race
        } else {
            self._query_race(race_id).await
        }
    }

    pub async fn get_club(&self, club_id: i32) -> Club {
        if let Some(race) = self.caches.club.get(&club_id).await {
            race
        } else {
            self._query_club(club_id).await
        }
    }

    pub async fn get_race_registrations(&self, race_id: i32) -> Vec<Registration> {
        if let Some(registrations) = self.caches.regs.get(&race_id).await {
            registrations
        } else {
            let registrations = self._query_race_registrations(race_id).await;
            self.caches.regs.set(&race_id, &registrations).await;
            registrations
        }
    }

    pub async fn get_heats(&self, regatta_id: i32, filter: Option<String>) -> Vec<Heat> {
        let start = Instant::now();

        if let Some(filter) = filter {
            debug!("Found filter={filter}");
            let rows = self._execute_query(Heat::search(regatta_id, filter)).await;
            return Heat::from_rows(&rows);
        }

        // 1. try to get heats from cache
        if let Some(heats) = self.caches.heats.get(&regatta_id).await {
            debug!(
                "Getting heats of regatta {} from cache: {:?}",
                regatta_id,
                start.elapsed()
            );
            heats
        } else {
            // 2. read heats from DB
            let rows = self._execute_query(Heat::query_all(regatta_id)).await;
            let heats: Vec<Heat> = Heat::from_rows(&rows);

            // 3. store heats in cache
            self.caches.heats.set(&regatta_id, &heats).await;
            debug!("Query heats of regatta {} from DB: {:?}", regatta_id, start.elapsed());

            heats
        }
    }

    pub async fn get_heat_registrations(&self, heat_id: i32) -> Vec<HeatRegistration> {
        // 1. try to get heat_registrations from cache
        if let Some(heat_regs) = self.caches.heat_regs.get(&heat_id).await {
            heat_regs
        } else {
            self._query_heat_registrations(heat_id).await
        }
    }

    pub async fn get_participating_clubs(&self, regatta_id: i32) -> Vec<Club> {
        if let Some(clubs) = self.caches.part_clubs.get(&regatta_id).await {
            clubs
        } else {
            self._query_participating_clubs(regatta_id).await
        }
    }

    pub async fn get_club_registrations(&self, regatta_id: i32, club_id: i32) -> Vec<Registration> {
        if let Some(registrations) = self.caches.club_regs.get(&(regatta_id, club_id)).await {
            registrations
        } else {
            self._query_club_registrations(regatta_id, club_id).await
        }
    }

    async fn _query_heat_registrations(&self, heat_id: i32) -> Vec<HeatRegistration> {
        let start = Instant::now();
        let mut heat_regs: Vec<HeatRegistration> =
            HeatRegistration::query_all(heat_id, &mut self.pool.get().await).await;
        for heat_registration in &mut heat_regs {
            let crew = Crew::query_all(heat_registration.registration.id, &mut self.pool.get().await).await;
            heat_registration.registration.crew = Some(crew);
        }
        self.caches.heat_regs.set(&heat_id, &heat_regs).await;
        debug!("Query registrations of heat {} from DB: {:?}", heat_id, start.elapsed());
        heat_regs
    }

    async fn _query_participating_clubs(&self, regatta_id: i32) -> Vec<Club> {
        let start = Instant::now();
        let clubs = Club::query_participating(regatta_id, &mut self.pool.get().await).await;
        self.caches.part_clubs.set(&regatta_id, &clubs).await;
        debug!(
            "Query participating clubs of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        clubs
    }

    async fn _query_club_registrations(&self, regatta_id: i32, club_id: i32) -> Vec<Registration> {
        let start = Instant::now();
        let registrations = Registration::query_of_club(regatta_id, club_id, &mut self.pool.get().await).await;
        self.caches.club_regs.set(&(regatta_id, club_id), &registrations).await;
        debug!(
            "Query registrations of club {} for regatta {} from DB: {:?}",
            club_id,
            regatta_id,
            start.elapsed()
        );
        registrations
    }

    pub async fn query_scoring(&self, regatta_id: i32) -> Vec<Score> {
        let start = Instant::now();
        let scores = Score::query_all(regatta_id, &mut self.pool.get().await).await;
        debug!("Query scoring of regatta {} from DB: {:?}", regatta_id, start.elapsed());
        scores
    }

    pub async fn query_kiosk(&self, regatta_id: i32) -> Kiosk {
        let start = Instant::now();

        let finished = self._execute_query(Kiosk::query_finished(regatta_id)).await;
        let next = self._execute_query(Kiosk::query_next(regatta_id)).await;
        let finished_heats = Heat::from_rows(&finished);
        let next_heats: Vec<Heat> = Heat::from_rows(&next);

        let kiosk = Kiosk {
            finished: finished_heats,
            next: next_heats,
            running: Vec::with_capacity(0),
        };
        debug!("Query kiosk of regatta {} from DB: {:?}", regatta_id, start.elapsed());
        kiosk
    }

    async fn _query_regatta(&self, regatta_id: i32) -> Regatta {
        let start = Instant::now();
        let regatta = Regatta::query(regatta_id, &mut self.pool.get().await).await;
        self.caches.regatta.set(&regatta.id, &regatta).await;
        debug!("Query regatta {} from DB: {:?}", regatta_id, start.elapsed());
        regatta
    }

    async fn _query_race(&self, race_id: i32) -> Race {
        let start = Instant::now();
        let race: Race = Race::query_single(race_id, &mut self.pool.get().await).await;
        self.caches.race.set(&race.id, &race).await;
        debug!("Query race {} from DB: {:?}", race_id, start.elapsed());
        race
    }

    async fn _query_races(&self, regatta_id: i32) -> Vec<Race> {
        let start = Instant::now();
        let races = Race::query_all(regatta_id, &mut self.pool.get().await).await;
        self.caches.races.set(&regatta_id, &races).await;
        debug!("Query races of regatta {} from DB: {:?}", regatta_id, start.elapsed());
        races
    }

    async fn _query_race_registrations(&self, race_id: i32) -> Vec<Registration> {
        let start = Instant::now();
        let registrations = Registration::query_for_race(race_id, &mut self.pool.get().await).await;
        debug!("Query registrations of race {} from DB: {:?}", race_id, start.elapsed());
        registrations
    }

    async fn _query_club(&self, club_id: i32) -> Club {
        let start = Instant::now();
        let club = Club::query_single(club_id, &mut self.pool.get().await).await;
        self.caches.club.set(&club.id, &club).await;
        debug!("Query club {} from DB: {:?}", club_id, start.elapsed());
        club
    }

    async fn _execute_single_query(&self, query: Query<'_>) -> Row {
        let mut client = self.pool.get().await;

        query
            .query(&mut client)
            .await
            .unwrap()
            .into_row()
            .await
            .unwrap()
            .unwrap()
    }

    async fn _execute_query(&self, query: Query<'_>) -> Vec<Row> {
        let mut client = self.pool.get().await;

        query
            .query(&mut client)
            .await
            .unwrap()
            .into_first_result()
            .await
            .unwrap()
    }
}
