use super::{
    cache::Cache,
    model::{
        crew::Crew, heat::Heat, heat::HeatRegistration, heat::Kiosk, race::Race, regatta::Regatta,
        registration::Registration, score::Score, statistics::Statistics,
    },
    pool::PoolFactory,
    TiberiusPool,
};
use actix_web_lab::__reexports::tracing::info;
use log::{debug, trace};
use tiberius::{Query, Row};

pub struct Aquarius {
    cache: Cache,
    pool: TiberiusPool,
}

impl Aquarius {
    /// Create a new `Aquarius`.
    pub async fn new() -> Self {
        Aquarius {
            cache: Cache::new(),
            pool: PoolFactory::create_pool().await,
        }
    }

    pub async fn get_regattas(&self) -> Vec<Regatta> {
        debug!("Query all regattas from DB");
        let rows = self._execute_query(Regatta::query_all()).await;
        Regatta::from_rows(&rows)
    }

    /// Tries to get the regatta from the cache or database
    ///
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    pub async fn get_regatta(&self, regatta_id: i32) -> Regatta {
        // 1. try to get regatta from cache
        if let Some(regatta) = self.cache.get_regatta(regatta_id) {
            return regatta;
        }

        // 2. read regatta from DB
        debug!("Query regatta {} from DB", regatta_id);
        let row = self
            ._execute_single_query(Regatta::query_single(regatta_id))
            .await;
        let regatta = Regatta::from_row(&row);

        // 3. store regatta in cache
        self.cache.insert_regatta(&regatta).await;

        regatta
    }

    pub async fn get_races(&self, regatta_id: i32) -> Vec<Race> {
        // 1. try to get races from cache
        if let Some(races) = self.cache.get_races(regatta_id) {
            return races;
        }

        // 2. read races from DB
        debug!("Query races of regatta {} from DB", regatta_id);
        let rows = self._execute_query(Race::query_all(regatta_id)).await;
        let races = Race::from_rows(&rows);

        // 3. store races in cache
        self.cache.insert_races(regatta_id, &races).await;

        races
    }

    pub async fn get_statistics(&self, regatta_id: i32) -> Statistics {
        debug!("Query statistics of regatta {} from DB", regatta_id);
        let row = self
            ._execute_single_query(Statistics::query(regatta_id))
            .await;

        Statistics::from_row(&row)
    }

    pub async fn get_race(&self, race_id: i32) -> Race {
        // 1. try to get race from cache
        if let Some(race) = self.cache.get_race(race_id) {
            return race;
        }

        // 2. read race from DB
        debug!("Query race {} from DB", race_id);
        let row = self
            ._execute_single_query(Race::query_single(race_id))
            .await;
        let race = Race::from_row(&row);

        // 3. store race in cache
        self.cache.insert_race(&race).await;

        race
    }

    pub async fn get_registrations(&self, race_id: i32) -> Vec<Registration> {
        // 1. try to get registrations from cache
        if let Some(race_regs) = self.cache.get_registrations(race_id) {
            return race_regs;
        }

        // 2. read registrations from DB
        debug!("Query registrations of race {} from DB", race_id);
        let rows = self._execute_query(Registration::query_all(race_id)).await;
        let mut registrations: Vec<Registration> = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut registration = Registration::from_row(row);

            let crew_rows = self._execute_query(Crew::query_all(registration.id)).await;
            let crews = Crew::from_rows(&crew_rows);

            registration.crew = Some(crews);
            trace!("{:?}", registration);
            registrations.push(registration);
        }

        // 3. store registrations in cache
        self.cache
            .insert_registrations(race_id, &registrations)
            .await;

        registrations
    }

    pub async fn get_heats(&self, regatta_id: i32, filter: Option<String>) -> Vec<Heat> {
        if let Some(filter) = filter {
            info!("Found filter={filter}");
            let rows = self._execute_query(Heat::search(regatta_id, filter)).await;
            return Heat::from_rows(&rows);
        }

        // 1. try to get heats from cache
        if let Some(heats) = self.cache.get_heats(regatta_id) {
            return heats;
        }

        // 2. read heats from DB
        debug!("Query heats of regatta {} from DB", regatta_id);
        let rows = self._execute_query(Heat::query_all(regatta_id)).await;
        let heats: Vec<Heat> = Heat::from_rows(&rows);

        // 3. store heats in cache
        self.cache.insert_heats(regatta_id, &heats).await;

        heats
    }

    pub async fn get_kiosk(&self, regatta_id: i32) -> Kiosk {
        let finished = self._execute_query(Kiosk::query_finished(regatta_id)).await;
        let next = self._execute_query(Kiosk::query_next(regatta_id)).await;
        let finished_heats = Heat::from_rows(&finished);
        let next_heats: Vec<Heat> = Heat::from_rows(&next);

        Kiosk {
            finished: finished_heats,
            next: next_heats,
            running: Vec::with_capacity(0),
        }
    }

    pub async fn get_heat_registrations(&self, heat_id: i32) -> Vec<HeatRegistration> {
        // 1. try to get heat_registrations from cache
        if let Some(heat_regs) = self.cache.get_heat_regs(heat_id) {
            return heat_regs;
        }

        // 2. read heat_registrations from DB
        debug!("Query registrations of heat {} from DB", heat_id);
        let rows = self
            ._execute_query(HeatRegistration::query_all(heat_id))
            .await;
        let mut heat_regs: Vec<HeatRegistration> = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut heat_registration = HeatRegistration::from(row);

            let crew_rows = self
                ._execute_query(Crew::query_all(heat_registration.registration.id))
                .await;
            let crews = Crew::from_rows(&crew_rows);
            heat_registration.registration.crew = Some(crews);

            heat_regs.push(heat_registration);
        }

        // 3. store heat_registrations in cache
        self.cache.insert_heat_regs(heat_id, &heat_regs).await;

        heat_regs
    }

    pub async fn get_scoring(&self, regatta_id: i32) -> Vec<Score> {
        // 1. try to get heat_registrations from cache
        if let Some(scores) = self.cache.get_scores(regatta_id) {
            return scores;
        }

        // 2. read scores from DB
        debug!("Query scores of regatta {} from DB", regatta_id);
        let rows = self._execute_query(Score::query_all(regatta_id)).await;
        let scores = Score::from_rows(&rows);

        // 3. store scores in cache
        self.cache.insert_scores(regatta_id, &scores).await;

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
