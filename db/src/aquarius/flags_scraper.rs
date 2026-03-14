use ::scraper::{Html, Selector};
use ::std::{collections::HashMap, sync::OnceLock};
use ::tracing::warn;

const BASE_URL: &str = "https://verwaltung.rudern.de";
// downloaded from https://verwaltung.rudern.de/flags
const FLAGS_CONTENT: &str = include_str!("flags.html");

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
    let mut club_flags = HashMap::new();

    let document = Html::parse_document(FLAGS_CONTENT);
    if let Ok(a_selector) = Selector::parse(r#"a"#)
        && let Ok(img_selector) = Selector::parse(r#"img"#)
    {
        for a in document.select(&a_selector) {
            if let Some(href) = a.value().attr("href")
                && href.starts_with("/clubs/")
            {
                for img in a.select(&img_selector) {
                    if let Some(src) = img.value().attr("src") {
                        let club_extern_id: i32 = href
                            .split('/')
                            .next_back()
                            .unwrap_or_default()
                            .parse()
                            .unwrap_or_default();
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
    } else {
        warn!("Failed to parse selectors for flags scraper");
    }
    club_flags
}

#[cfg(test)]
mod tests {
    use crate::aquarius::flags_scraper::ClubFlag;

    #[tokio_shared_rt::test(shared)]
    async fn test_crawler() {
        let club_flags = ClubFlag::get(&11008);
        assert_eq!(
            club_flags.unwrap().flag_url,
            "https://verwaltung.rudern.de/uploads/clubs/fdd52f8c4b5b15538341ea3e9edb11c3_small.png".to_owned()
        );
    }
}
