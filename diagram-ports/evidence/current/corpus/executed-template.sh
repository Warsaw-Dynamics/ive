#!/usr/bin/env bash
# Template only: controller runs after final stable state/tooling integration and fresh release build.
set -euo pipefail
diagram_workspace=${1:-/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports}
diagram_binary=${2:-/mnt/data/Projects/ive/target/release/diagram-viewer}
diagram_oracle=${3:-/tmp/ive-diagram-oracles.JwN5H5}
diagram_evidence=$(mktemp -d /tmp/ive-diagram-final-poststate.XXXXXX)
diagram_workspace=$(cd "$diagram_workspace" && pwd -P)
test -z "$(git -C "$diagram_workspace" status --porcelain=v1)"
python3 - "$diagram_workspace" "$diagram_evidence" <<'PY'
import sys, pathlib, subprocess, json, hashlib
w=pathlib.Path(sys.argv[1]); root=pathlib.Path(sys.argv[2])
repo=pathlib.Path(subprocess.check_output(['git','-C',str(w),'rev-parse','--show-toplevel'],text=True).strip())
head=subprocess.check_output(['git','-C',str(repo),'rev-parse','HEAD'],text=True).strip()
paths=subprocess.check_output(['git','-C',str(repo),'ls-tree','-r','--name-only',head,'diagram-ports/fixtures'],text=True).splitlines()
for lang in ['mermaid','plantuml']:(root/'input-native'/lang).mkdir(parents=True)
(root/'input').mkdir(); rows=[]; seen=set()
for rel in paths:
 if not rel.endswith(('.mmd','.puml')):continue
 name=pathlib.PurePosixPath(rel).name; language=pathlib.PurePosixPath(rel).parts[2]
 if name in seen:raise SystemExit('same-filename collision: '+name)
 seen.add(name); data=subprocess.check_output(['git','-C',str(repo),'show',head+':'+rel])
 (root/'input'/name).write_bytes(data); (root/'input-native'/language/name).write_bytes(data)
 rows.append({'path':rel,'file':name,'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)})
if not rows:raise SystemExit('committed fixture enumeration is empty')
(root/'snapshot.json').write_text(json.dumps({'head':head,'sourceSnapshot':rows},indent=2)+'\n')
(root/'head.txt').write_text(head+'\n')
print(json.dumps({'root':str(root),'head':head,'cases':len(rows),'sourceBytes':sum(x['bytes'] for x in rows)}))
PY
sha256sum "$diagram_binary" "$diagram_workspace/tools/oracles.mjs" "$diagram_workspace/tools/native-corpus.mjs" > "$diagram_evidence/executables-sha256.txt"
node "$diagram_workspace/tools/oracles.mjs" --corpus "$diagram_evidence/input" "$diagram_evidence/output" --oracle-root "$diagram_oracle" > "$diagram_evidence/original-run.log" 2>&1
node "$diagram_workspace/tools/native-corpus.mjs" export --fixtures "$diagram_evidence/input-native" --output "$diagram_evidence/native" --viewer "$diagram_binary" --original "$diagram_evidence/output/admission.json" > "$diagram_evidence/native-run.log" 2>&1
node "$diagram_workspace/tools/native-corpus.mjs" check "$diagram_evidence/native" > "$diagram_evidence/native-closure.log" 2>&1
mkdir "$diagram_evidence/repeat-input"
for diagram_case in architecture.mmd gantt.mmd git.mmd ganttbase.puml git.puml sequence.puml; do
  cp "$diagram_evidence/input/$diagram_case" "$diagram_evidence/repeat-input/$diagram_case"
done
node "$diagram_workspace/tools/oracles.mjs" --corpus "$diagram_evidence/repeat-input" "$diagram_evidence/repeat-output" --oracle-root "$diagram_oracle" > "$diagram_evidence/repeat-run.log" 2>&1
python3 /tmp/ive-task4b-original-rehearsal-verify.py "$diagram_evidence" "$diagram_evidence/native" > "$diagram_evidence/verification.log"
sha256sum "$diagram_evidence/output/admission.json" "$diagram_evidence/native/admission.json" "$diagram_evidence/repeat-output/admission.json" "$diagram_evidence/comparison.json" "$diagram_evidence/verification.json" > "$diagram_evidence/manifests-sha256.txt"
printf 'Fresh post-state evidence: %s\n' "$diagram_evidence"
# Record actual Java/dot/browser/timezone/fonts/config; inspect SVG error candidates and native captures separately.
# This template does not build a binary, execute the full gates, produce COVERAGE.md, or claim final compatibility.
