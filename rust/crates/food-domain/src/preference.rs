//! Member-scoped taste and dislike preferences (PRD v3 §8). Kept per member and never
//! pre-averaged into a household value: `MVP-023` aggregates them at planning time (PRD
//! §9.10), and averaging here would destroy the dimension it needs.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PreferenceError {
    #[error("{field} must not be empty or whitespace")]
    Empty { field: &'static str },
    #[error("unknown sentiment {0:?}")]
    UnknownSentiment(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sentiment {
    Like,
    Dislike,
}

impl Sentiment {
    pub const ALL: [Self; 2] = [Self::Like, Self::Dislike];

    /// The persisted token, snake_case as every other vocabulary here is.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Like => "like",
            Self::Dislike => "dislike",
        }
    }

    /// Matches exactly and does not trim, as the sibling vocabularies do.
    pub fn parse(raw: &str) -> Result<Self, PreferenceError> {
        Self::ALL
            .into_iter()
            .find(|s| s.as_str() == raw)
            .ok_or_else(|| PreferenceError::UnknownSentiment(raw.to_owned()))
    }
}

/// One thing a member likes or dislikes. `subject` is free text: the taste vocabulary is
/// open-ended in a way the restriction vocabulary is not, so there is nothing to close it to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberPreference {
    sentiment: Sentiment,
    subject: String,
}

impl MemberPreference {
    /// Trims, and rejects blank: a preference with no subject scores nothing.
    pub fn new(sentiment: Sentiment, subject: impl Into<String>) -> Result<Self, PreferenceError> {
        let subject = subject.into();
        let trimmed = subject.trim();
        if trimmed.is_empty() {
            return Err(PreferenceError::Empty { field: "subject" });
        }
        Ok(Self {
            sentiment,
            subject: trimmed.to_owned(),
        })
    }

    pub fn sentiment(&self) -> Sentiment {
        self.sentiment
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

/// One member's preferences, in first-seen order and deduplicated on the whole
/// `(sentiment, subject)` pair. A member who both likes and dislikes the same subject keeps
/// two entries: this layer records what it is told, and adjudicating the contradiction is
/// `MVP-023`'s scoring decision, not storage's.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemberPreferences(Vec<MemberPreference>);

impl MemberPreferences {
    pub fn new(items: impl IntoIterator<Item = MemberPreference>) -> Self {
        let mut kept: Vec<MemberPreference> = Vec::new();
        for item in items {
            if !kept.contains(&item) {
                kept.push(item);
            }
        }
        Self(kept)
    }

    pub fn preferences(&self) -> &[MemberPreference] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hard-coded rather than mapped from `ALL`: a test that mirrors the production
    /// constant cannot catch an addition to it.
    #[test]
    fn every_sentiment_round_trips_its_token() {
        assert!(!Sentiment::ALL.is_empty());
        for sentiment in Sentiment::ALL {
            assert_eq!(Sentiment::parse(sentiment.as_str()).unwrap(), sentiment);
        }
        assert_eq!(Sentiment::ALL.map(Sentiment::as_str), ["like", "dislike"]);
    }

    #[test]
    fn an_unknown_sentiment_token_is_rejected() {
        for raw in ["", "loves", "Like", " like", "hate"] {
            assert_eq!(
                Sentiment::parse(raw).unwrap_err(),
                PreferenceError::UnknownSentiment(raw.to_owned()),
                "{raw:?} must not parse"
            );
        }
    }

    #[test]
    fn a_blank_subject_is_rejected() {
        for raw in ["", "   ", "\t\n"] {
            assert_eq!(
                MemberPreference::new(Sentiment::Like, raw).unwrap_err(),
                PreferenceError::Empty { field: "subject" },
                "{raw:?} must not be a preference"
            );
        }
    }

    #[test]
    fn the_subject_is_trimmed() {
        let p = MemberPreference::new(Sentiment::Dislike, "  olives  ").unwrap();
        assert_eq!(p.subject(), "olives");
        assert_eq!(p.sentiment(), Sentiment::Dislike);
    }

    /// The dedup key is the `(sentiment, subject)` pair. Storage records what it is told and
    /// does not adjudicate a member who both likes and dislikes the same thing.
    #[test]
    fn the_same_subject_under_two_sentiments_is_two_entries() {
        let set = MemberPreferences::new([
            MemberPreference::new(Sentiment::Like, "olives").unwrap(),
            MemberPreference::new(Sentiment::Dislike, "olives").unwrap(),
        ]);
        assert_eq!(set.preferences().len(), 2);
    }

    #[test]
    fn the_set_keeps_first_seen_order_and_collapses_duplicates() {
        let set = MemberPreferences::new([
            MemberPreference::new(Sentiment::Dislike, "olives").unwrap(),
            MemberPreference::new(Sentiment::Like, "tofu").unwrap(),
            MemberPreference::new(Sentiment::Dislike, "olives").unwrap(),
        ]);
        assert_eq!(
            set.preferences(),
            [
                MemberPreference::new(Sentiment::Dislike, "olives").unwrap(),
                MemberPreference::new(Sentiment::Like, "tofu").unwrap(),
            ]
        );
    }

    #[test]
    fn the_empty_set_is_legal() {
        let set = MemberPreferences::new([]);
        assert!(set.preferences().is_empty());
        assert_eq!(set, MemberPreferences::default());
    }
}
