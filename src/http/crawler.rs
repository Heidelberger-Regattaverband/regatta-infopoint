use scraper::{Html, Selector};
use std::{collections::HashMap, fs};

const BASE_URL: &str = "https://verwaltung.rudern.de";

#[derive(Debug, PartialEq)]
pub struct ClubFlag {
    flag_url: String,
    club_id: String,
}

pub fn load_club_flags() -> HashMap<String, ClubFlag> {
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
                        let club_id = href.split('/').last().unwrap();
                        let flag_url = BASE_URL.to_owned() + src;
                        println!("{} - {}", club_id, flag_url);
                        club_flags.insert(
                            club_id.to_owned(),
                            ClubFlag {
                                flag_url,
                                club_id: club_id.to_owned(),
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
    use crate::http::crawler::load_club_flags;

    #[actix_web::test]
    async fn test_crawler() {
        let club_flags = load_club_flags();
        assert_eq!(
            club_flags.get("40028").unwrap().flag_url,
            "https://verwaltung.rudern.de/uploads/clubs/797f9c9bea3af22290a395b0ff3afa3f_small.png".to_owned()
        );
    }
}
