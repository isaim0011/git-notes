use serde::{Deserialize, Serialize};
use std::fmt;

/// The three built-in note namespaces. Extensible via `Custom`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Namespace {
    Comments,
    Review,
    Todos,
    Custom(String),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_builtin_without_prefix() {
        assert_eq!(Namespace::from_str("comments"), Namespace::Comments);
        assert_eq!(Namespace::from_str("review"), Namespace::Review);
        assert_eq!(Namespace::from_str("todos"), Namespace::Todos);
    }

    #[test]
    fn test_from_str_builtin_with_prefix() {
        assert_eq!(Namespace::from_str("refs/notes/comments"), Namespace::Comments);
        assert_eq!(Namespace::from_str("refs/notes/review"), Namespace::Review);
        assert_eq!(Namespace::from_str("refs/notes/todos"), Namespace::Todos);
    }

    #[test]
    fn test_from_str_custom() {
        assert_eq!(
            Namespace::from_str("bugs"),
            Namespace::Custom("bugs".to_string())
        );
        assert_eq!(
            Namespace::from_str("refs/notes/bugs"),
            Namespace::Custom("bugs".to_string())
        );
    }

    #[test]
    fn test_ref_path_and_display() {
        let cases = vec![
            (Namespace::Comments, "refs/notes/comments", "comments"),
            (Namespace::Review, "refs/notes/review", "review"),
            (Namespace::Todos, "refs/notes/todos", "todos"),
            (Namespace::Custom("my-notes".to_string()), "refs/notes/my-notes", "my-notes"),
        ];

        for (ns, expected_ref, expected_display) in cases {
            assert_eq!(ns.ref_path(), expected_ref);
            assert_eq!(ns.to_string(), expected_display);
            assert_eq!(Namespace::from_str(expected_ref), ns);
            assert_eq!(Namespace::from_str(expected_display), ns);
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
