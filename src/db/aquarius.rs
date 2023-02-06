use super::{
    cache::Cache,
    model::{
        heat::Heat, heat::HeatRegistration, race::Race, statistics::Statistics, Crew, Regatta,
        Registration, Score,
    },
    pool::PoolFactory,
    TiberiusPool,
};
use anyhow::{Ok, Result};
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

    pub async fn get_regattas(&self) -> Result<Vec<Regatta>> {
        debug!("Query all regattas from DB");
        let rows = self._execute_query(Regatta::query_all()).await;
        let mut regattas: Vec<Regatta> = Vec::with_capacity(rows.len());
        for row in &rows {
            let regatta = Regatta::from(row);
            self.cache.insert_regatta(&regatta).await;
            trace!("{:?}", regatta);
            regattas.push(regatta);
        }
        Ok(regattas)
    }

    /// Tries to get the regatta from the cache.
    ///
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    pub async fn get_regatta(&self, regatta_id: i32) -> Result<Regatta> {
        // 1. try to get regatta from cache
        if let Some(regatta) = self.cache.get_regatta(regatta_id) {
            return Ok(regatta);
        }

        // 2. read regatta from DB
        debug!("Query regatta {} from DB", regatta_id);
        let mut client = self.pool.get().await?;
        let row = Regatta::query_single(regatta_id)
            .query(&mut client)
            .await?
            .into_row()
            .await?
            .unwrap();
        let regatta = Regatta::from(&row);

        // 3. store regatta in cache
        self.cache.insert_regatta(&regatta).await;

        Ok(regatta)
    }

    pub async fn get_races(&self, regatta_id: i32) -> Result<Vec<Race>> {
        // 1. try to get races from cache
        if let Some(races) = self.cache.get_races(regatta_id) {
            return Ok(races);
        }

        // 2. read races from DB
        debug!("Query races of regatta {} from DB", regatta_id);
        let rows = self._execute_query(Race::query_all(regatta_id)).await;
        let mut races: Vec<Race> = Vec::with_capacity(rows.len());
        for row in &rows {
            let race = Race::from(row);
            trace!("{:?}", race);
            races.push(race);
        }

        // 3. store races in cache
        self.cache.insert_races(regatta_id, &races).await;

        Ok(races)
    }

    pub async fn get_statistics(&self, regatta_id: i32) -> Result<Statistics> {
        let mut client = self.pool.get().await?;

        debug!("Query statistics of regatta {} from DB", regatta_id);
        let row = Statistics::query_all(regatta_id)
            .query(&mut client)
            .await?
            .into_row()
            .await?
            .unwrap();
        let stats = Statistics::from(&row);

        Ok(stats)
    }

    pub async fn get_race(&self, race_id: i32) -> Result<Race> {
        // 1. try to get race from cache
        if let Some(race) = self.cache.get_race(race_id) {
            return Ok(race);
        }

        // 2. read race from DB
        let mut client = self.pool.get().await?;
        debug!("Query race {} from DB", race_id);
        let row = Race::query_single(race_id)
            .query(&mut client)
            .await?
            .into_row()
            .await?
            .unwrap();
        let race = Race::from(&row);

        // 3. store race in cache
        self.cache.insert_race(&race).await;

        Ok(race)
    }

    pub async fn get_registrations(&self, race_id: i32) -> Result<Vec<Registration>> {
        // 1. try to get registrations from cache
        if let Some(race_regs) = self.cache.get_registrations(race_id) {
            return Ok(race_regs);
        }

        // 2. read registrations from DB
        debug!("Query registrations of race {} from DB", race_id);
        let rows = self._execute_query(Registration::query_all(race_id)).await;
        let mut registrations: Vec<Registration> = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut registration = Registration::from(row);

            let crew_rows = self._execute_query(Crew::query_all(registration.id)).await;
            let mut crews: Vec<Crew> = Vec::with_capacity(crew_rows.len());
            for crew_row in &crew_rows {
                crews.push(Crew::from(crew_row));
            }
            registration.crew = Option::Some(crews);
            trace!("{:?}", registration);
            registrations.push(registration);
        }

        // 3. store registrations in cache
        self.cache
            .insert_registrations(race_id, &registrations)
            .await;

        Ok(registrations)
    }

    pub async fn get_heats(&self, regatta_id: i32) -> Result<Vec<Heat>> {
        // 1. try to get heats from cache
        if let Some(heats) = self.cache.get_heats(regatta_id) {
            return Ok(heats);
        }

        // 2. read heats from DB
        debug!("Query heats of regatta {} from DB", regatta_id);
        let rows = self._execute_query(Heat::query_all(regatta_id)).await;
        let mut heats: Vec<Heat> = Vec::with_capacity(rows.len());
        for row in &rows {
            let heat = Heat::from(row);
            trace!("{:?}", heat);
            heats.push(heat);
        }

        // 3. store heats in cache
        self.cache.insert_heats(regatta_id, &heats).await;

        Ok(heats)
    }

    pub async fn get_heat_registrations(&self, heat_id: i32) -> Result<Vec<HeatRegistration>> {
        // 1. try to get heat_registrations from cache
        if let Some(heat_regs) = self.cache.get_heat_regs(heat_id) {
            return Ok(heat_regs);
        }

        // 2. read heat_registrations from DB
        debug!("Query registrations of heat {} from DB", heat_id);
        let rows = self
            ._execute_query(HeatRegistration::query_all(heat_id))
            .await;
        let mut heat_regs: Vec<HeatRegistration> = Vec::with_capacity(rows.len());
        for row in &rows {
            let heat_registration = HeatRegistration::from(row);
            trace!("{:?}", heat_registration);
            heat_regs.push(heat_registration);
        }

        // 3. store heat_registrations in cache
        self.cache.insert_heat_regs(heat_id, &heat_regs).await;

        Ok(heat_regs)
    }

    pub async fn get_scoring(&self, regatta_id: i32) -> Result<Vec<Score>> {
        // 1. try to get heat_registrations from cache
        if let Some(scores) = self.cache.get_scores(regatta_id) {
            return Ok(scores);
        }

        // 2. read scores from DB
        debug!("Query scores of regatta {} from DB", regatta_id);
        let rows = self._execute_query(Score::query_all(regatta_id)).await;
        let mut scores: Vec<Score> = Vec::with_capacity(rows.len());
        for row in &rows {
            let score = Score::from(row);
            trace!("{:?}", score);
            scores.push(score);
        }

        // 3. store scores in cache
        self.cache.insert_scores(regatta_id, &scores).await;

        Ok(scores)
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
