//! What every program in an IVE terminal gets in its environment: the terminal type, and how to reach this IVE's
//! MCP server, so a Claude Code started there connects to the IVE it runs in (see README, "Drive IVE from an agent").

use std::sync::OnceLock;

pub const MCP_URL_VAR: &str = "IVE_MCP_URL";
/// Not `IVE_MCP_TOKEN`: an IVE started from the terminal would take that as its own token.
pub const MCP_TOKEN_VAR: &str = "IVE_MCP_KEY";

const TERMINAL: [(&str, &str); 3] = [("TERM", "xterm-256color"), ("COLORTERM", "truecolor"), ("TERM_PROGRAM", "ive")];

/// Set once this IVE serves MCP; terminals started before that, or with MCP off, go without.
static MCP: OnceLock<(String, String)> = OnceLock::new();

/// Gives terminals started from now on the MCP server's `url` and `token`; the first call wins.
pub fn announce_mcp(url: String, token: String) {
    let _ = MCP.set((url, token));
}

pub fn variables() -> Vec<(String, String)> {
    variables_with(MCP.get())
}

fn variables_with(mcp: Option<&(String, String)>) -> Vec<(String, String)> {
    let terminal = TERMINAL.iter().map(|(key, value)| ((*key).to_owned(), (*value).to_owned()));
    let mcp = mcp.into_iter().flat_map(|(url, token)| [(MCP_URL_VAR.to_owned(), url.clone()), (MCP_TOKEN_VAR.to_owned(), token.clone())]);
    terminal.chain(mcp).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_give_the_terminal_type_only_without_mcp() {
        // given
        let mcp = None;

        // when
        let variables = variables_with(mcp);

        // then
        let keys: Vec<_> = variables.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, ["TERM", "COLORTERM", "TERM_PROGRAM"]);
    }

    #[test]
    fn should_add_the_mcp_url_and_token() {
        // given
        let mcp = ("http://127.0.0.1:7461/mcp".to_owned(), "0123456789abcdef".to_owned());

        // when
        let variables = variables_with(Some(&mcp));

        // then
        assert_eq!(variables[3..], [
            (MCP_URL_VAR.to_owned(), "http://127.0.0.1:7461/mcp".to_owned()),
            (MCP_TOKEN_VAR.to_owned(), "0123456789abcdef".to_owned()),
        ]);
    }
}
