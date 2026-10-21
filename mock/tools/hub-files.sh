#!/bin/sh
# Writes data/files.gz, the file lists the hub (Ctrl+E) searches: the monorepo's tracked files as project
# "atlassian", internal-apps/forge-installer's as project "forge-installer", then a sample of the monorepo's ignored
# files (up to 150 per ignored folder or file git reports) as "atlassian excluded <how many there are>". Each list
# starts with "#<project>[ excluded <total>]"; each path is front-coded against the one before:
# "<shared prefix length> <rest>".
set -eu
cd "$(dirname "$0")/.."
repo=$(git rev-parse --show-toplevel)
front_code() {
  python3 -I -c 'import sys
prev = ""
for line in sys.stdin.read().split("\n"):
    if not line: continue
    n = 0
    while n < min(len(prev), len(line)) and prev[n] == line[n]: n += 1
    print(n, line[n:]); prev = line'
}
sample_ignored() {
  dirs=$(mktemp)
  git -C "$repo" ls-files -o -i --exclude-standard --directory > "$dirs"
  git -C "$repo" ls-files -o -i --exclude-standard | python3 -I -c 'import sys
roots = [d.rstrip("\n") for d in open(sys.argv[1]) if d.strip()]
paths = sorted(p.rstrip("\n") for p in sys.stdin if p.strip())
taken = {}
out = []
for p in paths:
    root = next((r for r in roots if p == r or (r.endswith("/") and p.startswith(r))), p)
    if taken.get(root, 0) < 150:
        taken[root] = taken.get(root, 0) + 1
        out.append(p)
print("#atlassian excluded", len(paths))
print("\n".join(out))' "$dirs"
  rm -f "$dirs"
}
{
  echo '#atlassian'
  git -C "$repo" ls-files | front_code
  echo '#forge-installer'
  git -C "$repo" ls-files internal-apps/forge-installer | sed 's|^internal-apps/forge-installer/||' | sed 's/^/0 /'
  sample_ignored | { read -r header; echo "$header"; front_code; }
} | gzip -9n > data/files.gz
