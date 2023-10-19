use scraper::{Html, Selector};
use std::{collections::HashMap, fs};

const BASE_URL: &str = "https://verwaltung.rudern.de";

struct ClubIcon {
    url: String,
    club_id: String,
}

struct Crawler {
}

impl Crawler {
    fn parse() -> HashMap<String, ClubIcon> {
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
                            let club_id = href.split("/").last().unwrap();
                            let url = BASE_URL.to_owned() + src;
                            println!("{} - {}", club_id, url);
                            club_flags.insert(
                                club_id.to_owned(),
                                ClubIcon {
                                    url,
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
}

#[cfg(test)]
mod tests {
    use super::Crawler;

    #[actix_web::test]
    async fn test_crawler() {
        let club_flags = Crawler::parse();
    }
}
