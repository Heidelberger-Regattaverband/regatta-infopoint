use log::debug;
use reqwest::Error;
use scraper::{Html, Selector};
use std::fs;

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
        // let body: String = reqwest::get(&self.url).await?.text().await?;
        let body: String = fs::read_to_string("static/webapp/flags.html").unwrap();
        debug!("body = {:?}", body);
        self.body = Some(body.clone());
        Ok(self)
    }

    fn parse(&self) {
        let document = Html::parse_document(self.body.as_ref().unwrap());
        let selector = Selector::parse(r#"a"#).unwrap();
        for title in document.select(&selector) {
            if let Some(href) = title.value().attr("href") {
                println!("{}", href);
            }
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
