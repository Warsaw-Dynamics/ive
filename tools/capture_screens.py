#!/usr/bin/env python3
"""Screenshots of a test IVE on a fixed fixture repository, to check that a change leaves IVE's look unchanged.

    tools/capture_screens.py <out-dir> [ive-binary] [--init <init.rn>]
                                                         capture (default binary: ./target/debug/ive), with
                                                         <init.rn> as the profile's init.rn if given
    tools/capture_screens.py --compare <a-dir> <b-dir>   per image, the number of differing pixels; exit 1 if any

The fixture repository is rebuilt at /tmp/ive-shots/fixture with fixed contents and commit dates, and the test
profile `agent-shots` is reset before each run, so two runs of the same build give the same pixels. Each run
captures every tab in the dark theme and again in the light theme, at 1280x800.
"""

import base64
import json
import os
import shutil
import signal
import subprocess
import sys
import time
import tomllib
import urllib.request
from pathlib import Path

PROFILE = "agent-shots"
FIXTURE = Path("/tmp/ive-shots/fixture")
CONFIG = Path.home() / ".config" / "ive" / PROFILE
DATE = "2024-01-02T03:04:05+00:00"
START_TIMEOUT = 60
SETTLE_SECONDS = 1.5

FILES = {
    "README.md": "# Fixture\n\nA repository for IVE screenshots.\n",
    "Cargo.toml": '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2024"\n',
    ".gitignore": "target/\n*.log\n",
    "src/main.rs": 'fn main() {\n    let greeting = "hello";\n    println!("{greeting}, world");\n}\n',
    "src/lib.rs": "/// Adds two numbers.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
}
# after the commit: a modified file, an untracked file, ignored files and folders, a node_modules folder
CHANGES = {
    "src/main.rs": 'fn main() {\n    let greeting = "hi";\n    println!("{greeting}, world!");\n}\n',
    "notes.txt": "untracked\n",
    "debug.log": "ignored\n",
    "target/debug/fixture": "binary\n",
    "node_modules/left-pad/index.js": "module.exports = s => s;\n",
}

# tab label in the file name -> saved tab (see src/saved_tabs.rs)
TABS = [
    ("file", {"kind": "file", "path": str(FIXTURE / "src/main.rs"), "scroll": {"top_line": 0.0, "left": 0.0}}),
    ("project", {"kind": "project-overview", "project": "fixture", "section": "Worktrees"}),
    ("worktree-git", {"kind": "worktree-overview", "checkout": str(FIXTURE), "section": "Git"}),
    ("diff", {"kind": "diff", "checkout": str(FIXTURE), "request": {
        "dir": str(FIXTURE), "path": list(b"src/main.rs"), "source": {"kind": "uncommitted"}}}),
    ("git-log", {"kind": "git-log", "project": "fixture", "view": "Log"}),
    ("settings", {"kind": "settings", "page": "Appearance"}),
]


def git(*args):
    env = {**os.environ, "GIT_AUTHOR_DATE": DATE, "GIT_COMMITTER_DATE": DATE, "GIT_AUTHOR_NAME": "Fixture",
           "GIT_AUTHOR_EMAIL": "fixture@example.com", "GIT_COMMITTER_NAME": "Fixture",
           "GIT_COMMITTER_EMAIL": "fixture@example.com"}
    subprocess.run(["git", "-C", str(FIXTURE), *args], check=True, env=env, capture_output=True)


def write_files(files):
    for name, text in files.items():
        path = FIXTURE / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)


def build_fixture():
    shutil.rmtree(FIXTURE, ignore_errors=True)
    FIXTURE.mkdir(parents=True)
    git("init", "-q", "-b", "main")
    write_files(FILES)
    git("add", "-A")
    git("commit", "-q", "-m", "Initial commit")
    write_files(CHANGES)


def state_toml():
    tabs = "".join(f"\n[[tabs.tabs]]\n{toml_inline_fields(tab)}" for _, tab in TABS)
    return f"""version = 2
recent = []

[[projects]]
path = "{FIXTURE}"
id = "fixture"
expanded = true
expanded_checkouts = ["{FIXTURE}"]
last_checkout = "{FIXTURE}"

[tabs]
active = 0
{tabs}
[window]
x = 0.0
y = 0.0
width = 1280.0
height = 800.0
mode = "windowed"
"""


def toml_inline_fields(tab):
    return "".join(f"{key} = {toml_value(value)}\n" for key, value in tab.items())


def toml_value(value):
    if isinstance(value, dict):
        return "{ " + ", ".join(f"{key} = {toml_value(item)}" for key, item in value.items()) + " }"
    return json.dumps(value)


def reset_profile(theme):
    shutil.rmtree(CONFIG, ignore_errors=True)
    CONFIG.mkdir(parents=True)
    (CONFIG / "state.toml").write_text(state_toml())
    (CONFIG / "settings.toml").write_text(f'theme = "{theme}"\n')


class Mcp:
    def __init__(self, port, token):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self.headers = {"Authorization": f"Bearer {token}", "Content-Type": "application/json",
                        "Accept": "application/json, text/event-stream"}
        response = self.post({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "capture", "version": "1"}}})
        session = response.headers.get("Mcp-Session-Id")
        if session:
            self.headers["Mcp-Session-Id"] = session
        self.post({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def post(self, body):
        request = urllib.request.Request(self.url, json.dumps(body).encode(), self.headers)
        return urllib.request.urlopen(request, timeout=30)

    def call(self, tool, arguments=None):
        response = self.post({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                              "params": {"name": tool, "arguments": arguments or {}}})
        text = response.read().decode()
        data = [line[5:].strip() for line in text.splitlines() if line.startswith("data:") and line[5:].strip()]
        message = json.loads(data[-1] if data else text)
        if "error" in message:
            raise RuntimeError(f"{tool}: {message['error']}")
        return message["result"]["content"]


def wait_for_mcp(process):
    deadline = time.time() + START_TIMEOUT
    while time.time() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"IVE exited with {process.returncode} before its MCP server came up")
        path = CONFIG / "mcp.toml"
        if path.exists():
            config = tomllib.loads(path.read_text())
            try:
                return Mcp(config["port"], config["token"])
            except OSError:
                pass
        time.sleep(0.5)
    raise RuntimeError(f"no MCP server from IVE within {START_TIMEOUT} s")


def end_terminals(binary):
    """Ends the test profile's hosted terminals, which outlive its IVE and would show in the next run's tree."""
    env = {**os.environ, "IVE_PTY_DIR": str(Path(os.environ["XDG_RUNTIME_DIR"]) / "ive-pty" / PROFILE)}
    listed = subprocess.run([binary, "--pty-host", "ls"], env=env, capture_output=True, text=True).stdout
    for session in (line.split()[0] for line in listed.splitlines() if line.strip()):
        subprocess.run([binary, "--pty-host", "kill", session], env=env, capture_output=True)


def capture_theme(binary, theme, out, init):
    end_terminals(binary)
    reset_profile(theme)
    if init is not None:
        shutil.copyfile(init, CONFIG / "init.rn")
    process = subprocess.Popen([binary, "--profile", PROFILE, "--background"], stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL, start_new_session=True)
    try:
        mcp = wait_for_mcp(process)
        time.sleep(SETTLE_SECONDS * 2)
        tabs = json.loads(mcp.call("list_tabs")[0]["text"])
        if len(tabs) != len(TABS):
            raise RuntimeError(f"IVE restored {len(tabs)} of {len(TABS)} tabs; is {CONFIG / 'state.toml.bad'} there?")
        for (label, _), tab in zip(TABS, tabs):
            mcp.call("activate_tab", {"tab": tab["id"]})
            time.sleep(SETTLE_SECONDS)
            parts = mcp.call("screenshot")
            image = next((part for part in parts if part["type"] == "image"), None)
            if image is None:
                raise RuntimeError(f"no screenshot of {theme}-{label}: {parts}")
            (out / f"{theme}-{label}.png").write_bytes(base64.b64decode(image["data"]))
    finally:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()


def capture(out, binary, init=None):
    out.mkdir(parents=True, exist_ok=True)
    build_fixture()
    for theme in ("dark", "light"):
        capture_theme(binary, theme, out, init)
    print(f"{len(list(out.glob('*.png')))} screenshots in {out}")


def compare(a, b):
    from PIL import Image, ImageChops
    differing = 0
    for path in sorted(a.glob("*.png")):
        other = b / path.name
        if not other.exists():
            print(f"{path.name}: missing in {b}")
            differing += 1
            continue
        first, second = Image.open(path).convert("RGB"), Image.open(other).convert("RGB")
        if first.size != second.size:
            print(f"{path.name}: size {first.size} vs {second.size}")
            differing += 1
            continue
        count = sum(1 for pixel in ImageChops.difference(first, second).getdata() if pixel != (0, 0, 0))
        print(f"{path.name}: {count}")
        differing += count
    return 1 if differing else 0


def main(args):
    if args[:1] == ["--compare"] and len(args) == 3:
        return compare(Path(args[1]), Path(args[2]))
    init = None
    if len(args) >= 3 and args[-2] == "--init":
        init, args = Path(args[-1]), args[:-2]
    if len(args) in (1, 2) and not args[0].startswith("-"):
        capture(Path(args[0]), args[1] if len(args) == 2 else "./target/debug/ive", init)
        return 0
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
