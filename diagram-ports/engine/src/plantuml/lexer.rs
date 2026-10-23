//! Bounded lexical replacement for MIT PlantUML NameAndCodeParser and arrow command grammars.
#[derive(Clone, Debug)]
pub(super) struct Token {
    pub text: String,
    pub quoted: bool,
}

pub(super) fn arrow_at(rest: &str) -> Option<&str> {
    if !["--", "->", "..", "<-", "<|", "o-", "o.", "*-", "*.", "<."]
        .iter()
        .any(|p| rest.starts_with(p))
    {
        return None;
    }
    let end = rest
        .find(|ch: char| !matches!(ch, '-' | '.' | '<' | '>' | '|' | 'o' | '*'))
        .unwrap_or(rest.len());
    Some(&rest[..end])
}
