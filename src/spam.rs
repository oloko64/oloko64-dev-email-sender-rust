use std::{fmt, sync::LazyLock};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use regex::Regex;
use serde_json::json;

use crate::utils::EmailBody;

/// Gmail address with 2+ dots in the local part, e.g. `sux.a.j.up.iji.z.4.0.7@gmail.com`.
/// Gmail ignores dots, so bots use them to generate "new" addresses from the same inbox.
static DOTTED_GMAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^[a-z0-9]+(\.[a-z0-9]+){2,}@gmail\.com$").expect("Invalid regex")
});

/// A single "word" of 12 to 30 ASCII letters, e.g. `wokSJigoCELNFDVKDe`.
static LETTERS_ONLY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z]{12,30}$").expect("Invalid regex"));

/// Minimum uppercase letters (ignoring the first character) for a text to count as random noise.
/// Keeps ordinary single words like `Collaboration` from matching.
const MIN_INNER_UPPERCASE: usize = 3;

/// Domain used by GrowGanic (AI SEO blog-writing SaaS) for contact-form outreach.
const GROWGANIC_DOMAIN: &str = "growganic.io";

/// Known spam patterns. When a request matches one, the API pretends to be down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpamPattern {
    /// Random mixed-case letters as subject and body, sent from a dotted Gmail address.
    RandomGmailGibberish,
    /// Outreach from GrowGanic (AI SEO blog tool), sent from a `growganic.io` address.
    GrowGanicOutreach,
}

impl SpamPattern {
    const ALL: &[SpamPattern] = &[
        SpamPattern::RandomGmailGibberish,
        SpamPattern::GrowGanicOutreach,
    ];

    /// Returns the first enabled pattern the body matches, if any.
    pub fn detect(body: &EmailBody) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .filter(|pattern| pattern.enabled())
            .find(|pattern| pattern.matches(body))
    }

    /// Whether the pattern is active. Disabled patterns are skipped by `detect`.
    const fn enabled(self) -> bool {
        match self {
            SpamPattern::RandomGmailGibberish => true,
            // Ready to use, but off for now. Set to `true` if they keep sending messages.
            SpamPattern::GrowGanicOutreach => false,
        }
    }

    fn matches(self, body: &EmailBody) -> bool {
        match self {
            SpamPattern::RandomGmailGibberish => {
                DOTTED_GMAIL.is_match(&body.contact)
                    && is_random_letters(&body.subject)
                    && is_random_letters(&body.body)
            }
            SpamPattern::GrowGanicOutreach => is_from_domain(&body.contact, GROWGANIC_DOMAIN),
        }
    }

    /// The response sent to the client: looks like the API Gateway reply when the backend is down.
    pub fn fake_response(self) -> Response {
        match self {
            SpamPattern::RandomGmailGibberish | SpamPattern::GrowGanicOutreach => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "message": "Service Unavailable" })),
            )
                .into_response(),
        }
    }
}

impl fmt::Display for SpamPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

fn is_random_letters(text: &str) -> bool {
    LETTERS_ONLY.is_match(text)
        && text
            .chars()
            .skip(1)
            .filter(char::is_ascii_uppercase)
            .count()
            >= MIN_INNER_UPPERCASE
}

/// True if `contact` is an email address at `domain` or one of its subdomains.
fn is_from_domain(contact: &str, domain: &str) -> bool {
    let Some((_, contact_domain)) = contact.trim().rsplit_once('@') else {
        return false;
    };
    let contact_domain = contact_domain.to_ascii_lowercase();
    contact_domain == domain
        || contact_domain
            .strip_suffix(domain)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(contact: &str, subject: &str, body: &str) -> EmailBody {
        EmailBody {
            contact: contact.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
        }
    }

    #[test]
    fn detects_known_samples() {
        let samples = [
            (
                "sux.a.j.up.iji.z.4.0.7@gmail.com",
                "wokSJigoCELNFDVKDe",
                "eWqeYypKnGbMflloYhVf",
            ),
            (
                "at.ebi.jo5.1.2@gmail.com",
                "YujReeJsEyNrMAyZgKwSD",
                "rckrRIhUolJMnkIl",
            ),
            (
                "nawos.o.w.569@gmail.com",
                "xNdgmoaiDDuFRGvtC",
                "qGCEvAmKWfKSouoFKL",
            ),
        ];

        for (contact, subject, message) in samples {
            assert_eq!(
                SpamPattern::detect(&body(contact, subject, message)),
                Some(SpamPattern::RandomGmailGibberish),
                "{contact} should be detected"
            );
        }
    }

    #[test]
    fn ignores_normal_message() {
        let b = body(
            "john.doe@gmail.com",
            "Job opportunity",
            "Hi, I would like to talk about a position.",
        );
        assert_eq!(SpamPattern::detect(&b), None);
    }

    #[test]
    fn ignores_dotted_gmail_with_normal_text() {
        let b = body(
            "j.o.h.n.doe@gmail.com",
            "Hello there",
            "Just wanted to say hi!",
        );
        assert_eq!(SpamPattern::detect(&b), None);
    }

    #[test]
    fn ignores_random_text_from_normal_contact() {
        let b = body(
            "someone@example.com",
            "wokSJigoCELNFDVKDe",
            "eWqeYypKnGbMflloYhVf",
        );
        assert_eq!(SpamPattern::detect(&b), None);
    }

    #[test]
    fn ignores_single_ordinary_words() {
        let b = body("a.b.c@gmail.com", "Collaboration", "Congratulations");
        assert_eq!(SpamPattern::detect(&b), None);
    }

    #[test]
    fn growganic_matches_its_domain() {
        for contact in [
            "hey@growganic.io",
            "  Hey@GrowGanic.IO ",
            "luca@mail.growganic.io",
        ] {
            assert!(
                SpamPattern::GrowGanicOutreach.matches(&body(contact, "Hi", "Hello")),
                "{contact} should match"
            );
        }
    }

    #[test]
    fn growganic_ignores_look_alikes() {
        for contact in [
            "x@notgrowganic.io",
            "growganic.io@gmail.com",
            "x@growganic.io.evil.com",
            "growganic.io",
        ] {
            assert!(
                !SpamPattern::GrowGanicOutreach.matches(&body(contact, "Hi", "Hello")),
                "{contact} should not match"
            );
        }
    }

    #[test]
    fn detect_follows_enabled_flag() {
        let b = body(
            "hey@growganic.io",
            "Question about your blog",
            "Hi, quick question: who looks after the blog?",
        );
        let expected = SpamPattern::GrowGanicOutreach
            .enabled()
            .then_some(SpamPattern::GrowGanicOutreach);
        assert_eq!(SpamPattern::detect(&b), expected);
    }

    #[test]
    fn fake_response_is_service_unavailable() {
        let response = SpamPattern::RandomGmailGibberish.fake_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
