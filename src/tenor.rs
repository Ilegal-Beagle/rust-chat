use std:: {
    env,
    error::Error
};
use serde::Deserialize;
use dotenvy;
use reqwest;

#[derive(Clone)]
pub struct TenorAPI {
    client: reqwest::Client,
    key: String,
    client_key: String,
}

impl TenorAPI {
    pub fn new() -> Result<TenorAPI, Box<dyn Error>> {
        let client = reqwest::Client::new();
        
        // make sure getting env var is successful, otherwise give an error
        dotenvy::dotenv().ok();
        let key = match env::var("TENOR_API_KEY"){
            Ok(key) => key,
            Err(e) => return Err(Box::new(e)),
        };

        Ok (Self {
            client: client,
            key: key,
            client_key: "Client Key".to_string(),
        })
    }

    // returns a vec of strings to the urls of featured gifs
    pub async fn featured(&mut self, limit: u32) -> Result<Vec<String>, Box<dyn Error>>{
        let mut gif_urls = Vec::<String>::new();
        let response = self.client.get(
            format!(
                "https://tenor.googleapis.com/v2/featured?key={}&client_key={}&limit={}",
                self.key, self.client_key, limit
        ))
            .send()
            .await?;

        let mut parsed = response.json::<TenorResponse>().await?;
        for result in parsed.results.iter_mut() {
            gif_urls.push(result.media_formats.gif.url.clone())
        }
        Ok(gif_urls)
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct TenorResponse {
    results: Vec<TenorResult>,
    next: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct TenorResult {
    media_formats: MediaFormats,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MediaFormats {
    gif: Gif,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Gif {
    url: String,
}