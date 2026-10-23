from pathlib import Path
from collections import deque
import hashlib, json, re

ROOT = Path('/mnt/data/Projects/ive/.worktree/diagram-tree-parallel/diagram-ports')

def explore(family, entry, goto):
    source = (ROOT / f'engine/src/plantuml/generated/{family}.rs').read_text()
    module = source.split(f'mod __parse__{entry} {{', 1)[1].split('\nmod __parse__', 1)[0]
    def table(name):
        raw = re.search(r'const '+name+r':.*?= &\[(.*?)\];', module, re.S).group(1)
        return [int(x) for x in re.findall(r'-?\d+', re.sub(r'//[^\n]*', '', raw))]
    action, eof = table('__ACTION'), table('__EOF_ACTION')
    terminals = len(action) // len(eof)
    body = module[module.index('fn __simulate_reduce'):module.index('pub struct')]
    reductions = {int(i): (int(p), int(n)) for i,p,n in re.findall(r'(\d+) => \{\s*__state_machine::SimulatedReduce::Reduce \{\s*states_to_pop: (\d+),\s*nonterminal_produced: (\d+)', body)}
    accept = int(re.search(r'(\d+) => __state_machine::SimulatedReduce::Accept', body).group(1))
    pending, seen = deque([(0,)]), {(0,)}
    live = consecutive = expected_max = diag = capacity_max = peak = 0
    while pending:
        original = pending.popleft()
        live = max(live, len(original))
        expected = 0
        for token in range(terminals + 1):
            stack = list(original)
            capacity = len(stack)
            steps = 0
            while True:
                diag = max(diag, len(stack))
                assert len(stack) <= 32, (family, entry, stack)
                value = eof[stack[-1]] if token == terminals else action[stack[-1]*terminals+token]
                if value >= 0: break
                steps += 1
                assert steps < 32
                reduction = -value - 1
                if reduction == accept: break
                pop, nt = reductions[reduction]
                if pop: del stack[-pop:]
                if len(stack) == capacity:
                    grown = max(8, capacity*2)
                    peak = max(peak, grown + capacity)
                    capacity = grown
                stack.append(goto(stack[-1], nt))
                capacity_max = max(capacity_max, capacity)
            consecutive = max(consecutive, steps)
            if value > 0:
                expected += token < terminals
                stack.append(value - 1)
                state = tuple(stack)
                if state not in seen:
                    seen.add(state)
                    pending.append(state)
        expected_max = max(expected_max, expected)
    return dict(family=family,entry=entry,sha256=hashlib.sha256(source.encode()).hexdigest(),states=len(eof),terminals=terminals,reachable=len(seen),max_live=live,max_diagnostic=diag,max_reductions=consecutive,max_expected=expected_max,max_diagnostic_capacity=capacity_max,max_diagnostic_growth_peak=peak)

def wbs_goto(state, nt):
    if nt == 9:
        return {3:49,4:54,5:59,6:70,7:75,8:80,9:83,10:87,11:94,12:99,13:106,14:113,15:119,16:125,17:128,18:130,19:147,20:149,21:151,22:155,23:157,24:159,25:160}.get(state,34)
    return {11:26,12:27,13:1}.get(nt,0)

print(json.dumps([
    explore('tree','Mindmap',lambda state,nt: {4:3,5:1,6:4,7:2}.get(nt,0)),
    explore('tree','Wbs',wbs_goto),
    explore('files','Files',lambda state,nt: {0:4,1:5,2:1,3:9,4:2,5:3,6:8}.get(nt,0)),
    explore('board','Board',lambda state,nt: {2:2,3:3,4:1,5:4}.get(nt,0)),
],indent=2))
