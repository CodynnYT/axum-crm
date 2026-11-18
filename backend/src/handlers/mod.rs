pub mod health;
pub mod auth;
pub mod users;
pub mod leads;
pub mod activities;
pub mod customers;
pub mod deals;
pub mod dashboard;
pub mod reports;

use serde::Deserialize;

// Optional search text and stage filters shared by list endpoints.
#[derive(Deserialize, Default)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub stage: Option<String>,
}

impl SearchQuery {
    pub fn pattern(&self) -> String {
        match self.q.as_deref().map(str::trim) {
            Some(text) if !text.is_empty() => format!("%{text}%"),
            _ => "%".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_search_matches_everything() {
        let query = SearchQuery{
            q: Some("     ".into()),
            stage: None,
        };
        assert_eq!(query.pattern(), "%");
    }

    #[test]
    fn search_text_is_wrapped() {
        let query = SearchQuery {
            q: Some("   acme  ".into()),
            stage: None
        };
        assert_eq!(query.pattern(), "%acme%");
    }

}

