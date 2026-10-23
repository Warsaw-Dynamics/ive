from pathlib import Path
import hashlib
import os
import re
import signal
import subprocess

viewer = '/mnt/data/Projects/ive/target/release/diagram-viewer'
fixtures = Path('/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/plantuml')
families = ['sequence', 'state', 'mindmap', 'wbs', 'files', 'board']

def run_checked(arguments, log, environment):
    with log.open('wb') as output:
        process = subprocess.Popen(arguments, stdout=output, stderr=subprocess.STDOUT, env=environment, start_new_session=True)
        try:
            result = process.wait(timeout=30)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            raise
    assert result == 0, (arguments, result, log)

for family in families:
    prefix = Path(f'/tmp/ive-parser-integration1-{family}')
    frame = prefix.with_suffix('.png')
    log = prefix.with_suffix('.log')
    run_checked([viewer, '--language', 'plantuml', '--example', f'plantuml-{family}', '--background', '--capture', str(frame)], log, os.environ.copy())
    metrics = re.search(r'native measurements (\d+) · deterministic fallbacks (\d+)', log.read_text())
    assert metrics and int(metrics.group(1)) > 0 and int(metrics.group(2)) == 0, (family, log)
    environment = os.environ.copy()
    environment['PATH'] = ''
    environment.pop('DISPLAY', None)
    environment.pop('WAYLAND_DISPLAY', None)
    svg = prefix.with_suffix('.svg')
    run_checked([viewer, '--language', 'plantuml', '--render', str(fixtures / f'{family}.puml'), '--output', str(svg)], prefix.with_suffix('.export.log'), environment)
    assert svg.stat().st_size > 0
    print(f'{family}: native={metrics.group(1)}, fallback=0, frame={hashlib.sha256(frame.read_bytes()).hexdigest()}, empty-PATH export={svg.stat().st_size}', flush=True)
