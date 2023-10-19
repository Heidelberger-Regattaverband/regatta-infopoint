use scraper::{Html, Selector};
use std::{collections::HashMap, fs};

const BASE_URL: &str = "https://verwaltung.rudern.de";

#[derive(Debug, PartialEq)]
pub struct ClubFlag {
    pub flag_url: String,
    pub club_id: i32,
}

pub fn load_club_flags() -> HashMap<i32, ClubFlag> {
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
                        let mut club_id: i32 = href.split('/').last().unwrap_or_default().parse().unwrap_or_default();
                        club_id -= 11000;
                        let flag_url = BASE_URL.to_owned() + src;
                        club_flags.insert(club_id, ClubFlag { flag_url, club_id });
                    }
                }
            }
        }
    }
    club_flags
}

#[cfg(test)]
mod tests {
    use crate::http::crawler::load_club_flags;

    #[actix_web::test]
    async fn test_crawler() {
        let club_flags = load_club_flags();
        assert_eq!(
            club_flags.get(&8).unwrap().flag_url,
            "https://verwaltung.rudern.de/uploads/clubs/fdd52f8c4b5b15538341ea3e9edb11c3_small.png".to_owned()
        );
    }
}
