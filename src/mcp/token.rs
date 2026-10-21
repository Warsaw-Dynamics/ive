use std::fmt;

/// Shorter tokens are too easy to guess for a server that can run shell commands.
const MIN_TOKEN_CHARS: usize = 16;
const GENERATED_TOKEN_BYTES: usize = 24;
pub const TOKEN_ENV: &str = "IVE_MCP_TOKEN";

/// Bearer token every MCP request must carry.
#[derive(Clone)]
pub struct Token {
    value: String,
    /// from `IVE_MCP_TOKEN`, so the user knows it and it is never written down
    from_env: bool,
}

#[derive(Debug)]
pub enum TokenError {
    TooShort { chars: usize },
    NoRandomness(String),
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort { chars } => write!(f, "MCP token must be at least {MIN_TOKEN_CHARS} characters, got {chars}"),
            Self::NoRandomness(reason) => write!(f, "cannot generate an MCP token: {reason}"),
        }
    }
}

impl Token {
    /// `IVE_MCP_TOKEN`, when set.
    pub fn from_env() -> Option<Result<Self, TokenError>> {
        std::env::var(TOKEN_ENV).ok().map(|value| Self::checked(value, true))
    }

    /// One IVE generated earlier and kept.
    pub fn remembered(value: String) -> Result<Self, TokenError> {
        Self::checked(value, false)
    }

    pub fn random() -> Result<Self, TokenError> {
        let mut bytes = [0u8; GENERATED_TOKEN_BYTES];
        getrandom::fill(&mut bytes).map_err(|error| TokenError::NoRandomness(error.to_string()))?;
        let value = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        Ok(Self { value, from_env: false })
    }

    fn checked(value: String, from_env: bool) -> Result<Self, TokenError> {
        let chars = value.chars().count();
        if chars < MIN_TOKEN_CHARS {
            return Err(TokenError::TooShort { chars });
        }
        Ok(Self { value, from_env })
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn is_from_env(&self) -> bool {
        self.from_env
    }

    /// Compares in constant time, so response timing does not reveal how much of a guess was right.
    pub fn matches_header(&self, header: &str) -> bool {
        let Some(given) = header.strip_prefix("Bearer ") else {
            return false;
        };
        let (given, expected) = (given.as_bytes(), self.value.as_bytes());
        given.len() == expected.len() && given.iter().zip(expected).fold(0u8, |diff, (a, b)| diff | (a ^ b)) == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_accept_only_exact_bearer_token() {
        // given
        let token = Token::remembered("0123456789abcdef".to_owned()).expect("long enough");

        // when
        let checks = ["Bearer 0123456789abcdef", "Bearer 0123456789abcdeX", "0123456789abcdef", "Bearer 0123"]
            .map(|header| token.matches_header(header));

        // then
        assert_eq!(checks, [true, false, false, false]);
    }

    #[test]
    fn should_reject_short_token() {
        // given
        let value = "short".to_owned();

        // when
        let token = Token::remembered(value);

        // then
        assert!(matches!(token, Err(TokenError::TooShort { chars: 5 })));
    }

    #[test]
    fn should_generate_distinct_hex_tokens() {
        // given
        let (first, second) = (Token::random().expect("random"), Token::random().expect("random"));

        // when
        let values = [first.value(), second.value()];

        // then
        assert!(values[0] != values[1] && values.iter().all(|value| value.len() == 48 && value.chars().all(|c| c.is_ascii_hexdigit())));
    }
}
