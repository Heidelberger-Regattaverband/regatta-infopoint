use super::{cache::Cache, model::Regatta, pool::create_pool, TiberiusPool};
use crate::db::{
    model::{self, Heat, HeatRegistration, Score},
    utils::Column,
};
use anyhow::{Ok, Result};
use log::{debug, trace};
use tiberius::{time::chrono::NaiveDateTime, Row};

const REGATTAS_QUERY: &str = "SELECT * FROM Event e";

const REGATTA_QUERY: &str = "SELECT * FROM Event e WHERE e.Event_ID = @P1";

const HEATS_QUERY: &str =
    "SELECT DISTINCT c.*, o.Offer_RaceNumber, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, ag.*, r.* \
    FROM Comp AS c \
    FULL OUTER JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK \
    FULL OUTER JOIN AgeClass AS ag ON o.Offer_AgeClass_ID_FK = ag.AgeClass_ID \
    FULL OUTER JOIN CompReferee AS cr ON cr.CompReferee_Comp_ID_FK = c.Comp_ID \
    FULL OUTER JOIN Referee AS r ON r.Referee_ID = cr.CompReferee_Referee_ID_FK \
    WHERE c.Comp_Event_ID_FK = @P1 \
    ORDER BY c.Comp_DateTime ASC";

const HEAT_REGISTRATION_QUERY: &str =
    "SELECT	DISTINCT ce.*, e.Entry_Bib, e.Entry_BoatNumber, e.Entry_Comment, l.Label_Short, r.Result_Rank, r.Result_DisplayValue, r.Result_Delta \
    FROM CompEntries AS ce
    FULL OUTER JOIN Comp AS c ON ce.CE_Comp_ID_FK = c.Comp_ID
    FULL OUTER JOIN Entry AS e ON ce.CE_Entry_ID_FK = e.Entry_ID
    FULL OUTER JOIN EntryLabel AS el ON el.EL_Entry_ID_FK = e.Entry_ID
    FULL OUTER JOIN Label AS l ON el.EL_Label_ID_FK = l.Label_ID
    FULL OUTER JOIN Result AS r ON r.Result_CE_ID_FK = ce.CE_ID
    WHERE ce.CE_Comp_ID_FK = @P1 AND (r.Result_SplitNr = 64 OR r.Result_SplitNr IS NULL) \
      AND el.EL_RoundFrom <= c.Comp_Round AND c.Comp_Round <= el.EL_RoundTo";

const SCORES_QUERY: &str = "SELECT s.rank, s.points, c.Club_Name, c.Club_Abbr FROM HRV_Score s JOIN Club AS c ON s.club_id = c.Club_ID WHERE s.event_id = @P1 ORDER BY s.rank ASC";

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
        let mut client = self.pool.get().await.unwrap();

        debug!("Query regattas from DB");
        trace!("Execute query {}", HEATS_QUERY);
        let rows = client
            .query(REGATTAS_QUERY, &[])
            .await?
            .into_first_result()
            .await?;

        let mut regattas: Vec<Regatta> = Vec::with_capacity(rows.len());

        for row in &rows {
            let regatta = model::create_regatta(row);
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
        let regatta_opt = self.cache.get_regatta(regatta_id).await;
        if regatta_opt.is_some() {
            return Ok(regatta_opt.unwrap());
        }

        // 2. read regatta from DB
        debug!("Query regatta {} from DB", regatta_id);
        trace!("Execute query {}", REGATTA_QUERY);
        let mut client = self.pool.get().await.unwrap();
        let row = client
            .query(REGATTA_QUERY, &[&regatta_id])
            .await?
            .into_row()
            .await?
            .unwrap();
        let regatta = model::create_regatta(&row);

        // 3. store regatta in cache
        self.cache.insert_regatta(&regatta).await;

        Ok(regatta)
    }

    pub async fn get_heats(&self, regatta_id: i32) -> Result<Vec<Heat>> {
        // 1. try to get heats from cache
        let heats_opt = self.cache.get_heats(regatta_id).await;
        if heats_opt.is_some() {
            return Ok(heats_opt.unwrap());
        }

        // 2. read heats from DB
        debug!("Query heats of regatta {} from DB", regatta_id);
        trace!("Execute query {}", HEATS_QUERY);
        let mut client = self.pool.get().await.unwrap();
        let rows = client
            .query(HEATS_QUERY, &[&regatta_id])
            .await?
            .into_first_result()
            .await?;
        let mut heats: Vec<Heat> = Vec::with_capacity(rows.len());
        for row in &rows {
            let heat = model::create_heat(row);
            trace!("{:?}", heat);
            heats.push(heat);
        }

        // 3. store heats in cache
        self.cache.insert_heats(regatta_id, &heats).await;

        Ok(heats)
    }

    pub async fn get_heat_registrations(&self, heat_id: i32) -> Result<Vec<HeatRegistration>> {
        // 1. try to get heat_registrations from cache
        let opt = self.cache.get_heat_regs(heat_id).await;
        if opt.is_some() {
            return Ok(opt.unwrap());
        }

        // 2. read heat_registrations from DB
        debug!("Query registrations of heat {} from DB", heat_id);
        trace!("Execute query {}", HEAT_REGISTRATION_QUERY);
        let mut client = self.pool.get().await.unwrap();
        let rows = client
            .query(HEAT_REGISTRATION_QUERY, &[&heat_id])
            .await?
            .into_first_result()
            .await?;
        let mut heat_regs: Vec<HeatRegistration> = Vec::with_capacity(rows.len());
        for row in &rows {
            let heat_registration = model::create_heat_registration(row);
            trace!("{:?}", heat_registration);
            heat_regs.push(heat_registration);
        }

        // 3. store heat_registrations in cache
        self.cache.insert_heat_regs(heat_id, &heat_regs).await;

        Ok(heat_regs)
    }

    pub async fn get_scoring(&self, regatta_id: i32) -> Result<Vec<Score>> {
        // 1. try to get heat_registrations from cache
        let opt = self.cache.get_scores(regatta_id).await;
        if opt.is_some() {
            return Ok(opt.unwrap());
        }

        // 2. read scores from DB
        debug!("Query scores of regatta {} from DB", regatta_id);
        trace!("Execute query {}", SCORES_QUERY);
        let mut client = self.pool.get().await.unwrap();
        let rows = client
            .query(SCORES_QUERY, &[&regatta_id])
            .await?
            .into_first_result()
            .await?;
        let mut scores: Vec<Score> = Vec::with_capacity(rows.len());
        for row in &rows {
            let score = model::create_score(row);
            trace!("{:?}", score);
            scores.push(score);
        }

        // 3. store scores in cache
        self.cache.insert_scores(regatta_id, &scores).await;

        Ok(scores)
    }
}
