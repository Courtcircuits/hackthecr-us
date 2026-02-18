use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::Scraper;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSchool {
    #[serde(rename = "code_uai")]
    pub code_uai: String,
    #[serde(rename = "ndeg_siret")]
    pub ndeg_siret: Option<f64>,
    #[serde(rename = "type_d_etablissement")]
    pub type_d_etablissement: String,
    pub nom: String,
    pub sigle: Option<String>,
    pub statut: String,
    pub tutelle: Option<String>,
    pub universite: Option<String>,
    #[serde(rename = "boite_postale")]
    pub boite_postale: Option<String>,
    pub adresse: String,
    pub cp: f64,
    pub commune: String,
    pub telephone: String,
    #[serde(rename = "debut_portes_ouvertes")]
    pub debut_portes_ouvertes: Option<String>,
    #[serde(rename = "fin_portes_ouvertes")]
    pub fin_portes_ouvertes: Option<String>,
    #[serde(rename = "commentaires_portes_ouvertes")]
    pub commentaires_portes_ouvertes: Option<String>,
    #[serde(rename = "lien_site_onisep_fr")]
    pub lien_site_onisep_fr: String,
    #[serde(rename = "point_geo")]
    pub point_geo: PointGeo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointGeo {
    pub lon: f64,
    pub lat: f64,
}

impl Display for PointGeo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.lat, self.lon)
    }
}

#[derive(Debug, Deserialize)]
struct HeraultData {
    results: Vec<ApiSchool>,
}

#[derive(Debug)]
pub struct SchoolApiScraper {
    url: String,
}

impl SchoolApiScraper {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    async fn fetch_page(&self, offset: usize) -> Result<HeraultData, SchoolApiScraperError> {
        let url = format!("{}?limit=100&offset={}", self.url, offset);
        let body = reqwest::get(&url)
            .await
            .map_err(|_| SchoolApiScraperError::RequestFailed)?
            .text()
            .await
            .map_err(|_| SchoolApiScraperError::RequestFailed)?;

        serde_json::from_str(&body).map_err(|_| SchoolApiScraperError::ParsingFailed)
    }
}

#[derive(Debug)]
pub enum SchoolApiScraperError {
    RequestFailed,
    ParsingFailed,
}

impl Scraper<Vec<ApiSchool>> for SchoolApiScraper {
    type Failure = SchoolApiScraperError;

    async fn scrape(&self) -> Result<Vec<ApiSchool>, Self::Failure> {
        let (page1, page2) =
            tokio::try_join!(self.fetch_page(0), self.fetch_page(100))?;

        let schools = page1
            .results
            .into_iter()
            .chain(page2.results)
            .filter(|s| s.statut.contains("Public"))
            .collect();

        Ok(schools)
    }
}
