use super::{
    cache::{CacheTrait, Caches},
    model::{Crew, Heat, HeatRegistration, Kiosk, Race, Regatta, Registration, Score, Statistics, ToEntity},
    tiberius::{PoolFactory, TiberiusConnectionManager, TiberiusPool},
};
use bb8::PooledConnection;
use log::debug;
use std::time::Instant;
use tiberius::{Query, Row};

pub type AquariusClient<'a> = PooledConnection<'a, TiberiusConnectionManager>;

pub struct Aquarius {
    caches: Caches,
    pub pool: TiberiusPool,
}

impl Aquarius {
    /// Create a new `Aquarius`.
    pub async fn new() -> Self {
        Aquarius {
            caches: Caches::new(),
            pool: PoolFactory::create_pool().await,
        }
    }

    pub async fn get_regattas(&self) -> Vec<Regatta> {
        let start = Instant::now();
        let regattas = Regatta::query_all(&mut self.pool.get().await.unwrap()).await;
        debug!("Query all regattas from DB: {:?}", start.elapsed());

        regattas
    }

    /// Tries to get the regatta from the cache or database
    ///
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    pub async fn get_regatta(&self, regatta_id: i32) -> Regatta {
        let start = Instant::now();

        // 1. try to get regatta from cache
        if let Some(regatta) = self.caches.regatta.get(&regatta_id) {
            debug!("Getting regatta {} from cache: {:?}", regatta_id, start.elapsed());

            regatta
        } else {
            // 2. read regatta from DB
            let regatta = Regatta::query(regatta_id, &mut self.pool.get().await.unwrap()).await;

            // 3. store regatta in cache
            self.caches.regatta.set(&regatta.id, &regatta).await;

            debug!("Query regatta {} from DB: {:?}", regatta_id, start.elapsed());

            regatta
        }
    }

    pub async fn get_races(&self, regatta_id: i32) -> Vec<Race> {
        let start = Instant::now();

        // 1. try to get races from cache
        if let Some(races) = self.caches.races.get(&regatta_id) {
            debug!(
                "Getting races of regatta {} from cache: {:?}",
                regatta_id,
                start.elapsed()
            );
            races
        } else {
            // 2. read races from DB
            let rows = self._execute_query(Race::query_all(regatta_id)).await;
            let races = Race::from_rows(&rows);

            // 3. store races in cache
            self.caches.races.set(&regatta_id, &races).await;
            debug!("Query races of regatta {} from DB: {:?}", regatta_id, start.elapsed());

            races
        }
    }

    pub async fn get_statistics(&self, regatta_id: i32) -> Statistics {
        let start = Instant::now();
        let row = self._execute_single_query(Statistics::query(regatta_id)).await;
        let stats = row.to_entity();
        debug!(
            "Query statistics of regatta {} from DB: {:?}",
            regatta_id,
            start.elapsed()
        );
        stats
    }

    pub async fn get_race(&self, race_id: i32) -> Race {
        let start = Instant::now();

        if let Some(race) = self.caches.race.get(&race_id) {
            debug!("Getting race {} from cache:  {:?}ms", race_id, start.elapsed());
            race
        } else {
            // 2. read race from DB
            let row = self._execute_single_query(Race::query_single(race_id)).await;
            let race: Race = row.to_entity();

            // 3. store race in cache
            self.caches.race.set(&race.id, &race).await;

            debug!("Query race {} from DB: {:?}ms", race_id, start.elapsed());

            race
        }
    }

    pub async fn get_registrations(&self, race_id: i32) -> Vec<Registration> {
        let start = Instant::now();

        // 1. try to get registrations from cache
        if let Some(registrations) = self.caches.regs.get(&race_id) {
            debug!(
                "Getting registrations of race {} from cache: {:?}",
                race_id,
                start.elapsed()
            );
            registrations
        } else {
            // 2. read registrations from DB
            let rows = self._execute_query(Registration::query_all(race_id)).await;
            let mut registrations: Vec<Registration> = Vec::with_capacity(rows.len());
            for row in &rows {
                let mut registration: Registration = row.to_entity();

                let crew_rows = self._execute_query(Crew::query_all(registration.id)).await;
                registration.crew = Some(Crew::from_rows(&crew_rows));
                registrations.push(registration);
            }

            // 3. store registrations in cache
            self.caches.regs.set(&race_id, &registrations).await;
            debug!("Query registrations of race {} from DB: {:?}", race_id, start.elapsed());

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
        if let Some(heats) = self.caches.heats.get(&regatta_id) {
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

    pub async fn get_kiosk(&self, regatta_id: i32) -> Kiosk {
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

    pub async fn get_heat_registrations(&self, heat_id: i32) -> Vec<HeatRegistration> {
        let start = Instant::now();

        // 1. try to get heat_registrations from cache
        if let Some(heat_regs) = self.caches.heat_regs.get(&heat_id) {
            debug!(
                "Getting registrations of heat {} from cache: {:?}",
                heat_id,
                start.elapsed()
            );
            heat_regs
        } else {
            // 2. read heat_registrations from DB
            let rows = self._execute_query(HeatRegistration::query_all(heat_id)).await;
            let mut heat_regs: Vec<HeatRegistration> = Vec::with_capacity(rows.len());
            for row in &rows {
                let mut heat_registration: HeatRegistration = row.to_entity();

                let crew_rows = self
                    ._execute_query(Crew::query_all(heat_registration.registration.id))
                    .await;
                let crews = Crew::from_rows(&crew_rows);
                heat_registration.registration.crew = Some(crews);

                heat_regs.push(heat_registration);
            }

            // 3. store heat_registrations in cache
            self.caches.heat_regs.set(&heat_id, &heat_regs).await;
            debug!("Query registrations of heat {} from DB: {:?}", heat_id, start.elapsed());

            heat_regs
        }
    }

    pub async fn get_scoring(&self, regatta_id: i32) -> Vec<Score> {
        // 1. try to get heat_registrations from cache
        if let Some(scores) = self.caches.scores.get(&regatta_id) {
            return scores;
        }

        // 2. read scores from DB
        debug!("Query scores of regatta {} from DB", regatta_id);
        let rows = self._execute_query(Score::query_all(regatta_id)).await;
        let scores = Score::from_rows(&rows);

        // 3. store scores in cache
        self.caches.scores.set(&regatta_id, &scores).await;

        scores
    }

    async fn _execute_single_query(&self, query: Query<'_>) -> Row {
        let mut client = self.pool.get().await.unwrap();

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
        let mut client = self.pool.get().await.unwrap();

        query
            .query(&mut client)
            .await
            .unwrap()
            .into_first_result()
            .await
            .unwrap()
    }
}
