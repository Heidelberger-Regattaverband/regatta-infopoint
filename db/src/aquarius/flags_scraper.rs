use ::scraper::Html;
use ::scraper::Selector;
use ::std::collections::HashMap;
use ::std::sync::OnceLock;
use ::tracing::warn;

const BASE_URL: &str = "https://www.rudern.de/sites/default/files/styles/club_logo/public/images/vereine/";
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
    if let Ok(a_selector) = Selector::parse(r#"a"#) {
        for a in document.select(&a_selector) {
            if let Some(href) = a.value().attr("href")
                && href.starts_with("/clubs/")
            {
                let Some(club_extern_id) = href.split('/').next_back().and_then(|s| s.parse::<i32>().ok()) else {
                    warn!("Failed to parse club ID from href: {href}");
                    continue;
                };
                let flag_url = BASE_URL.to_owned() + club_extern_id.to_string().as_str() + ".png";
                club_flags.insert(
                    club_extern_id,
                    ClubFlag {
                        flag_url,
                        club_extern_id,
                    },
                );
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
