from pathlib import Path
from collections import deque
import hashlib, json, re

root = Path('/mnt/data/Projects/ive/.worktree/diagram-uml-parallel/diagram-ports')
source = (root/'engine/src/plantuml/generated/uml.rs').read_text()
def table(name):
    raw = re.search(r'const '+name+r':.*?= &\[(.*?)\];', source, re.S).group(1)
    return [int(value) for value in re.findall(r'-?\d+', re.sub(r'//[^\n]*', '', raw))]
action, eof = table('__ACTION'), table('__EOF_ACTION')
terminals = len(action)//len(eof)
body = source[source.index('fn __simulate_reduce'):source.index('pub struct')]
reductions = {int(index):(int(pop),int(nt)) for index,pop,nt in re.findall(r'(\d+) => \{\s*__state_machine::SimulatedReduce::Reduce \{\s*states_to_pop: (\d+),\s*nonterminal_produced: (\d+)', body)}
accept = int(re.search(r'(\d+) => __state_machine::SimulatedReduce::Accept', body).group(1))
goto_source = source[source.index('fn __goto'):source.index('const __TERMINAL')]
defaults, cases = {}, {}
for nt, match in re.findall(r'(\d+) => match state \{(.*?)\}', goto_source, re.S):
    cases[int(nt)] = {}
    for pattern,value in re.findall(r'([.\d| _=]+) => (\d+)', match):
        if pattern.strip() == '_': defaults[int(nt)] = int(value)
        else:
            for state in pattern.split('|'):
                if '..=' in state:
                    first,last=map(int,state.split('..='))
                    for entry in range(first,last+1): cases[int(nt)][entry]=int(value)
                else: cases[int(nt)][int(state)] = int(value)
simple = re.sub(r'\d+ => match state \{.*?\},', '', goto_source, flags=re.S)
defaults.update({int(nt):int(value) for nt,value in re.findall(r'(\d+) => (\d+)', simple)})
def goto(state, nt):
    return cases.get(nt, {}).get(state, defaults.get(nt, 0))

def explore(depth):
    pending, seen = deque([(0,)]), {(0,)}
    live = consecutive = expected_max = capacity_max = peak = 0
    while pending:
        original = pending.popleft()
        expected = 0
        for token in range(terminals + 1):
            stack = list(original)
            capacity, steps = len(stack), 0
            while True:
                live = max(live, len(stack))
                value = eof[stack[-1]] if token == terminals else action[stack[-1]*terminals+token]
                if value >= 0: break
                steps += 1
                assert steps < 32
                reduction = -value-1
                if reduction == accept: break
                pop, nt = reductions[reduction]
                if pop: del stack[-pop:]
                if len(stack) == capacity:
                    grown = max(8,capacity*2)
                    peak = max(peak,capacity+grown)
                    capacity = grown
                stack.append(goto(stack[-1],nt))
                capacity_max = max(capacity_max,capacity)
            consecutive = max(consecutive,steps)
            if value > 0:
                expected += token < terminals
                stack.append(value-1)
                live = max(live,len(stack))
                state = tuple(stack)
                if stack.count(2) <= depth and state not in seen:
                    seen.add(state)
                    pending.append(state)
        expected_max = max(expected_max,expected)
    assert live <= 2*depth+20
    return dict(depth=depth,reachable=len(seen),max_live=live,max_reductions=consecutive,max_expected=expected_max,max_diagnostic_capacity=capacity_max,max_diagnostic_growth_peak=peak)

print(json.dumps(dict(sha256=hashlib.sha256(source.encode()).hexdigest(),states=len(eof),terminals=terminals,proof=[explore(depth) for depth in [0,1,2,8,64,65]]), indent=2))
