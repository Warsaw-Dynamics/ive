#!/usr/bin/env bash
# Checks IVE's dependencies against deny.toml and rewrites THIRD-PARTY-NOTICES.md and assets/licenses/third-party.json
# from Cargo.lock.
# Run it after a dependency changes, and commit the notices with the change.
# Needs: cargo install --locked cargo-deny && cargo install --locked cargo-about --features cli
set -euo pipefail
cd "$(dirname "$0")/.."

for tool in deny about; do
    if ! cargo "$tool" --version >/dev/null 2>&1; then
        echo "cargo-$tool is missing: cargo install --locked cargo-$tool$([ "$tool" = about ] && echo ' --features cli')" >&2
        exit 1
    fi
done

cargo deny check
# IVE's own crates have no licence file of their own and are left out of the notices anyway
cargo about generate tools/third-party-notices.hbs -o THIRD-PARTY-NOTICES.md 2>&1 \
    | grep -v 'LicenseRef-WarsawDynamics-Proprietary has no license file' || true
# some licence texts come with CRLF line ends; the repository keeps LF
perl -pi -e 's/\r$//' THIRD-PARTY-NOTICES.md
echo "THIRD-PARTY-NOTICES.md: $(grep -c '^- \[' THIRD-PARTY-NOTICES.md) crate entries"
# the same crates and NOTICE's other software, for the Settings page Third-Party Software
cargo about generate --format json 2>/dev/null | tools/third_party_json.py
