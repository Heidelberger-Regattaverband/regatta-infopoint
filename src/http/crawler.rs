use scraper::{Html, Selector};
use std::{collections::HashMap, fs, sync::OnceLock};

const BASE_URL: &str = "https://verwaltung.rudern.de";

static CLUB_FLAGS: OnceLock<HashMap<i32, ClubFlag>> = OnceLock::new();

#[derive(Debug, PartialEq)]
pub struct ClubFlag {
    pub flag_url: String,
    pub club_extern_id: i32,
}

impl ClubFlag {
    pub fn get(id: &i32) -> Option<&ClubFlag> {
        CLUB_FLAGS.get_or_init(load_club_flags).get(id)
    }
}

fn load_club_flags() -> HashMap<i32, ClubFlag> {
    let body: String = fs::read_to_string("static/webapp/flags.html").unwrap();

    let document = Html::parse_document(&body);
    let selector = Selector::parse(r#"a"#).unwrap();
    let img_selector = Selector::parse(r#"img"#).unwrap();

    let mut club_flags = HashMap::new();

    for a in document.select(&selector) {
        if let Some(href) = a.value().attr("href") {
            if href.starts_with("/clubs/") {
                for img in a.select(&img_selector) {
                    if let Some(src) = img.value().attr("src") {
                        let club_extern_id: i32 =
                            href.split('/').last().unwrap_or_default().parse().unwrap_or_default();
                        let flag_url = BASE_URL.to_owned() + src;
                        club_flags.insert(
                            club_extern_id,
                            ClubFlag {
                                flag_url,
                                club_extern_id,
                            },
                        );
                    }
                }
            }
        }
    }
    club_flags
}

#[cfg(test)]
mod tests {
    use crate::http::crawler::ClubFlag;

    #[actix_web::test]
    async fn test_crawler() {
        let club_flags = ClubFlag::get(&11008);
        assert_eq!(
            club_flags.unwrap().flag_url,
            "https://verwaltung.rudern.de/uploads/clubs/fdd52f8c4b5b15538341ea3e9edb11c3_small.png".to_owned()
        );
    }
}
