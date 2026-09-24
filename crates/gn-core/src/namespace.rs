use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The three built-in note namespaces. Extensible via `Custom`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Namespace {
    Comments,
    Review,
    Todos,
    Custom(String),
}

impl From<Namespace> for String {
    fn from(ns: Namespace) -> Self {
        ns.to_string()
    }
}

impl FromStr for Namespace {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str(s))
    }
}

impl TryFrom<String> for Namespace {
    type Error = std::convert::Infallible;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(Self::from_str(&s))
    }
}

impl Namespace {
    /// Returns the full ref path for this namespace.
    pub fn ref_path(&self) -> String {
        match self {
            Namespace::Comments => "refs/notes/comments".to_string(),
            Namespace::Review => "refs/notes/review".to_string(),
            Namespace::Todos => "refs/notes/todos".to_string(),
            Namespace::Custom(name) => format!("refs/notes/{}", name),
        }
    }

    /// Parses a string into a Namespace. Will match built-in names if possible.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        // Strip the standard prefix if provided
        let stripped = s.strip_prefix("refs/notes/").unwrap_or(s);
        match stripped {
            "comments" => Namespace::Comments,
            "review" => Namespace::Review,
            "todos" => Namespace::Todos,
            other => Namespace::Custom(other.to_string()),
        }
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Namespace::Comments => write!(f, "comments"),
            Namespace::Review => write!(f, "review"),
            Namespace::Todos => write!(f, "todos"),
            Namespace::Custom(name) => write!(f, "{}", name),
        }
    }
}
