use super::{
    cache::Cache,
    model::{Heat, HeatRegistration, Race, Regatta, Registration, Score},
    pool::create_pool,
    TiberiusPool,
};
use anyhow::{Ok, Result};
use log::{debug, trace};

pub struct Aquarius {
    cache: Cache,
    pool: TiberiusPool,
}

impl Aquarius {
    /// Create a new `Aquarius`.
    pub async fn new() -> Self {
        Aquarius {
            cache: Cache::new(),
            pool: create_pool().await,
        }
    }

    pub async fn get_regattas(&self) -> Result<Vec<Regatta>> {
        let mut client = self.pool.get().await?;

        debug!("Query all regattas from DB");
        let rows = Regatta::query_all()
            .query(&mut client)
            .await?
            .into_first_result()
            .await?;
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
        let regatta_opt = self.cache.get_regatta(regatta_id);
        if regatta_opt.is_some() {
            return Ok(regatta_opt.unwrap());
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
        let regatta_opt = self.cache.get_races(regatta_id);
        if regatta_opt.is_some() {
            return Ok(regatta_opt.unwrap());
        }

        // 2. read races from DB
        debug!("Query races for regatta {} from DB", regatta_id);
        let mut client = self.pool.get().await?;
        let rows = Race::query_all(regatta_id)
            .query(&mut client)
            .await?
            .into_first_result()
            .await?;
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

    pub async fn get_race(&self, race_id: i32) -> Result<Race> {
        let mut client = self.pool.get().await?;

        debug!("Query race {} from DB", race_id);
        let row = Race::query_single(race_id)
            .query(&mut client)
            .await?
            .into_row()
            .await?
            .unwrap();
        let race = Race::from(&row);

        Ok(race)
    }

    pub async fn get_race_registrations(&self, race_id: i32) -> Result<Vec<Registration>> {
        debug!("Query registrations for race {} from DB", race_id);
        let mut client = self.pool.get().await?;
        let rows = Registration::query_all(race_id)
            .query(&mut client)
            .await?
            .into_first_result()
            .await?;
        let mut registrations: Vec<Registration> = Vec::with_capacity(rows.len());
        for row in &rows {
            let registration = Registration::from(row);
            trace!("{:?}", registration);
            registrations.push(registration);
        }

        Ok(registrations)
    }

    pub async fn get_heats(&self, regatta_id: i32) -> Result<Vec<Heat>> {
        // 1. try to get heats from cache
        let heats_opt = self.cache.get_heats(regatta_id);
        if heats_opt.is_some() {
            return Ok(heats_opt.unwrap());
        }

        // 2. read heats from DB
        debug!("Query heats of regatta {} from DB", regatta_id);
        let mut client = self.pool.get().await?;
        let rows = Heat::query_all(regatta_id)
            .query(&mut client)
            .await?
            .into_first_result()
            .await?;
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
        let opt = self.cache.get_heat_regs(heat_id);
        if opt.is_some() {
            return Ok(opt.unwrap());
        }

        // 2. read heat_registrations from DB
        debug!("Query registrations of heat {} from DB", heat_id);
        let mut client = self.pool.get().await?;
        let rows = HeatRegistration::query_all(heat_id)
            .query(&mut client)
            .await?
            .into_first_result()
            .await?;
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
        let opt = self.cache.get_scores(regatta_id);
        if opt.is_some() {
            return Ok(opt.unwrap());
        }

        // 2. read scores from DB
        debug!("Query scores of regatta {} from DB", regatta_id);
        //trace!("Execute query {}", Score::query_all());
        let mut client = self.pool.get().await?;

        let query = Score::query_all(regatta_id);
        let rows = query
            .query(&mut client)
            .await
            .unwrap()
            .into_first_result()
            .await
            .unwrap();

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
}
