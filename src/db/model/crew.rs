use std::collections::HashMap;

use crate::db::{
    aquarius::AquariusClient,
    model::{utils, Athlete, ToEntity},
    tiberius::RowColumn,
};
use log::debug;
use serde::Serialize;
use tiberius::{Query, Row};

use super::Registration;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Crew {
    id: i32,

    /// position in the boat
    pos: u8,

    /// is the cox
    cox: bool,

    /// the athlete
    athlete: Athlete,

    round_from: i16,

    round_to: i16,
}

impl ToEntity<Crew> for Row {
    fn to_entity(&self) -> Crew {
        Crew {
            id: self.get_column("Crew_ID"),
            pos: self.get_column("Crew_Pos"),
            cox: self.get_column("Crew_IsCox"),
            athlete: self.to_entity(),
            round_from: self.get_column("Crew_RoundFrom"),
            round_to: self.get_column("Crew_RoundTo"),
        }
    }
}

impl Crew {
    pub async fn query_all<'a>(registration_id: i32, round: i16, client: &mut AquariusClient<'_>) -> Vec<Crew> {
        let mut query = Query::new(
            "SELECT Crew.*, Athlet.*, Club.Club_ID, Club.Club_Abbr, Club.Club_UltraAbbr, Club.Club_City
            FROM Crew
            JOIN Athlet ON Crew_Athlete_ID_FK = Athlet_ID
            JOIN Club   ON Athlet_Club_ID_FK  = Club_ID
            WHERE Crew_Entry_ID_FK = @P1 AND Crew_RoundFrom <= @P2 AND @P2 <= Crew_RoundTo
            ORDER BY Crew_pos ASC",
        );
        query.bind(registration_id);
        query.bind(round);
        let stream = query.query(client).await.unwrap();
        let crew = utils::get_rows(stream).await;
        crew.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn query_for_all_regs<'a>(
        registrations: &Vec<Registration>,
        round: i16,
        client: &mut AquariusClient<'_>,
    ) {
        let query_str = _create_query_string(registrations);

        let mut query = Query::new(query_str);
        query.bind(round);
        for reg in registrations {
            query.bind(reg.id);
        }

        let stream = query.query(client).await.unwrap();
        let rows = utils::get_rows(stream).await;
        let crews: Vec<Crew> = rows.into_iter().map(|row| row.to_entity()).collect();
        HashMap::from_iter(crews.iter());
    }
}

fn _create_query_string(registrations: &Vec<Registration>) -> String {
    let mut query_str = "SELECT Crew.*, Athlet.*, Club.Club_ID, Club.Club_Abbr, Club.Club_UltraAbbr, Club.Club_City
            FROM Crew
            JOIN Athlet ON Crew_Athlete_ID_FK = Athlet_ID
            JOIN Club   ON Athlet_Club_ID_FK  = Club_ID
            WHERE Crew_RoundFrom <= @P1 AND @P1 <= Crew_RoundTo AND "
        .to_owned();

    let mut index = 2;
    for _reg in registrations {
        query_str = query_str + "Crew_Entry_ID_FK = @P" + &index.to_string();
        if index <= registrations.len() {
            query_str += " OR ";
        }
        index += 1;
    }
    query_str += " ORDER BY Crew_pos ASC";

    debug!("Created query: {}", query_str);
    query_str
}
