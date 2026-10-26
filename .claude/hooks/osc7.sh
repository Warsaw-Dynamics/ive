#!/usr/bin/env bash
# Claude Code PostToolUse hook (EnterWorktree|ExitWorktree): reports the session's new working directory to its
# terminal with OSC 7, so terminals that follow it (IVE, WezTerm, kitty, iTerm2) know where Claude now works.
# Hooks run without a controlling terminal, and Claude Code drops OSC 7 from a hook's terminalSequence, so this
# writes to Claude's own terminal device. It never fails the tool call: anything missing means nothing is sent.
set -u
# bytes, not characters: the path is percent-encoded byte by byte
export LC_ALL=C

# the first "cwd" of the hook's JSON input is the top-level one; JSON escapes only \, " and control characters
input=$(cat)
[[ $input =~ \"cwd\"[[:space:]]*:[[:space:]]*\"(([^\"\\]|\\.)*)\" ]] || exit 0
cwd=${BASH_REMATCH[1]}
cwd=${cwd//\\\"/\"}
cwd=${cwd//\\\//\/}
cwd=${cwd//\\\\/\\}
[ -n "$cwd" ] && [ -n "${CLAUDE_PID:-}" ] || exit 0

tty=$(ps -o tty= -p "$CLAUDE_PID" 2>/dev/null | tr -d ' ')
case "$tty" in '' | '?' | '??') exit 0 ;; esac
device="/dev/$tty"
[ -w "$device" ] || exit 0

path=
for ((i = 0; i < ${#cwd}; i++)); do
    byte=${cwd:i:1}
    case "$byte" in
        [A-Za-z0-9/._~-]) path+=$byte ;;
        *) printf -v hex '%%%02X' "'$byte" && path+=$hex ;;
    esac
done
printf '\033]7;file://%s%s\033\\' "$(uname -n)" "$path" >"$device" 2>/dev/null
exit 0
