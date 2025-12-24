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
    pub async fn featured(&mut self, limit: u32) -> Result<Vec<TenorGif>, Box<dyn Error>>{
        let mut gif_urls = Vec::<TenorGif>::new();
        let response = self.client.get(
            format!(
                "https://tenor.googleapis.com/v2/featured?key={}&client_key={}&limit={}",
                self.key, self.client_key, limit
        ))
            .send()
            .await?;

        let text = response.text().await?;
        // println!("{:?}", text);

        let parsed = serde_json::from_str::<TenorResponse>(&text)?;
        for result in parsed.results {
            gif_urls.push( TenorGif {
                id: result.id.clone(),
                url: result.media_formats.gif.url.clone(),
                tinygif_url: result.media_formats.tinygif.url.clone(),
            });
        }
        Ok(gif_urls)
    }

    pub async fn search(&mut self, search: String, limit: u32) -> Result<Vec<TenorGif>, Box<dyn Error>> {
        let mut gif_urls = Vec::<TenorGif>::new();
        let response = self.client.get(
            format!(
                "https://tenor.googleapis.com/v2/search?q={}&key={}&client_key={}&limit={}",
                search, self.key, self.client_key, limit
            )
        )
            .send()
            .await?;

        let text = response.text().await?;
        // println!("{:?}", text);

        let parsed = serde_json::from_str::<TenorResponse>(&text)?;
        for result in parsed.results {
            gif_urls.push( TenorGif {
                id: result.id.clone(),
                url: result.media_formats.gif.url.clone(),
                tinygif_url: result.media_formats.tinygif.url.clone()
            });
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
    id: String,
    media_formats: MediaFormats,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MediaFormats {
    gif: Gif,
    tinygif: TinyGif,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Gif {
    url: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct TinyGif {
    url: String,
}

pub struct TenorGif {
    pub id: String,
    pub url: String,
    pub tinygif_url: String,
}