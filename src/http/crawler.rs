use log::debug;
use reqwest::Error;
use scraper::{Html, Selector};

struct ClubIcon {
    url: String,
    club_id: i32,
}

struct Crawler {
    url: String,
    body: Option<String>,
}

impl Crawler {
    pub fn new(url: String) -> Self {
        Crawler { url, body: None }
    }

    async fn fetch(&mut self) -> Result<&Self, Error> {
        let body = reqwest::get(&self.url).await?.text().await?;
        debug!("body = {:?}", body);
        self.body = Some(body.clone());
        Ok(self)
    }

    fn parse(&self) {
        let document = Html::parse_document(self.body.as_ref().unwrap());
        let selector = Selector::parse(r#"table > tbody > tr > td > a"#).unwrap();
        for title in document.select(&selector) {
            println!("{}", title.value().attr("href").expect("href not found").to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Crawler;

    #[actix_web::test]
    async fn test_crawler() {
        let mut crawler = Crawler::new("https://verwaltung.rudern.de/flags".to_owned());
        let _content = crawler.fetch().await.unwrap().parse();
    }
}
