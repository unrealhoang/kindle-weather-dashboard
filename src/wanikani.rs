use anyhow::{Context, bail};
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct WanikaniClient {
    http: Client,
    token: Option<String>,
}

impl WanikaniClient {
    pub fn new() -> Self {
        let token = std::env::var("WANIKANI_API_TOKEN").ok();

        Self {
            http: Client::new(),
            token,
        }
    }

    pub async fn fetch_pending_kanji(&self, limit: usize) -> anyhow::Result<Vec<WanikaniKanji>> {
        let token = self
            .token
            .as_ref()
            .context("WANIKANI_API_TOKEN is not set")?;

        let mut assignments: Vec<Assignment> = Vec::new();
        let mut next_url = Some(
            "https://api.wanikani.com/v2/assignments?subject_types=kanji&available_before=now"
                .to_string(),
        );

        while let Some(url) = next_url {
            if assignments.len() >= limit {
                break;
            }

            let resp: AssignmentsResponse = self
                .http
                .get(&url)
                .bearer_auth(token)
                .send()
                .await
                .context("failed to call WaniKani assignments API")?
                .error_for_status()
                .context("WaniKani assignments API returned an error")?
                .json()
                .await
                .context("failed to decode WaniKani assignments response")?;

            assignments.extend(resp.data);
            next_url = resp.pages.next_url;
        }

        let subject_ids: Vec<String> = assignments
            .into_iter()
            .take(limit)
            .map(|a| a.data.subject_id.to_string())
            .collect();

        if subject_ids.is_empty() {
            bail!("no pending kanji available");
        }

        let ids_query = subject_ids.join(",");
        let subjects_url = format!("https://api.wanikani.com/v2/subjects?ids={ids_query}");
        let subjects: SubjectsResponse = self
            .http
            .get(subjects_url)
            .bearer_auth(token)
            .send()
            .await
            .context("failed to call WaniKani subjects API")?
            .error_for_status()
            .context("WaniKani subjects API returned an error")?
            .json()
            .await
            .context("failed to decode WaniKani subjects response")?;

        let mut kanji = Vec::new();
        for subject in subjects.data.into_iter().take(limit) {
            let character = subject.data.characters.unwrap_or_else(|| "?".to_string());
            let meaning = subject
                .data
                .meanings
                .iter()
                .find(|m| m.primary)
                .or_else(|| subject.data.meanings.first())
                .map(|m| m.meaning.clone())
                .unwrap_or_else(|| "(no meaning)".to_string());

            kanji.push(WanikaniKanji { character, meaning });
        }

        Ok(kanji)
    }
}

#[derive(Debug)]
pub struct WanikaniKanji {
    pub character: String,
    pub meaning: String,
}

#[derive(Deserialize)]
struct AssignmentsResponse {
    data: Vec<Assignment>,
    pages: Pagination,
}

#[derive(Deserialize)]
struct Pagination {
    next_url: Option<String>,
}

#[derive(Deserialize)]
struct Assignment {
    data: AssignmentData,
}

#[derive(Deserialize)]
struct AssignmentData {
    subject_id: u64,
}

#[derive(Deserialize)]
struct SubjectsResponse {
    data: Vec<Subject>,
}

#[derive(Deserialize)]
struct Subject {
    data: SubjectData,
}

#[derive(Deserialize)]
struct SubjectData {
    characters: Option<String>,
    meanings: Vec<Meaning>,
}

#[derive(Deserialize)]
struct Meaning {
    meaning: String,
    primary: bool,
}
