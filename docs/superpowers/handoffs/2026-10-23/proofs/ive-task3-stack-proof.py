from pathlib import Path
from collections import deque
import re
import hashlib

root = Path('/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports')
source = (root / 'engine/src/plantuml/generated/preamble.rs').read_text()
print('generated_sha256', hashlib.sha256(source.encode()).hexdigest())
def table(label):
    raw = re.search(r'const ' + label + r':.*?= &\[(.*?)\];', source, re.S).group(1)
    return [int(x) for x in re.findall(r'-?\d+', re.sub(r'//[^\n]*', '', raw))]
action, eof = table('__ACTION'), table('__EOF_ACTION')
terminals = len(action) // len(eof)
body = source[source.index('fn __simulate_reduce'):source.index('pub struct')]
reductions = {int(index): (int(pop), int(nt)) for index, pop, nt in re.findall(
    r'(\d+) => \{\s*__state_machine::SimulatedReduce::Reduce \{\s*states_to_pop: (\d+),\s*nonterminal_produced: (\d+)', body)}
accept = int(re.search(r'(\d+) => __state_machine::SimulatedReduce::Accept', body).group(1))
def goto(state, nt):
    if nt == 6: return 49 if state == 9 else 58
    if nt == 7: return {7:11, 12:13}.get(state, 10)
    if nt == 19: return 42 if state == 8 else 32
    return {4:2, 5:27, 8:9, 9:50, 10:5, 11:6, 12:15, 13:1, 14:14,
        15:16, 16:12, 17:51, 18:7, 20:8, 21:17, 22:3, 23:37}.get(nt, 0)
pending, seen = deque([(0,)]), {(0,)}
max_states = max_diagnostic = max_steps = max_expected = max_capacity = max_peak = 0
while pending:
    original = pending.popleft()
    max_states = max(max_states, len(original))
    expected = 0
    for token in range(terminals + 1):
        stack = list(original)
        capacity = len(stack)
        steps = 0
        while True:
            max_diagnostic = max(max_diagnostic, len(stack))
            assert len(stack) < 64
            value = eof[stack[-1]] if token == terminals else action[stack[-1] * terminals + token]
            if value >= 0: break
            steps += 1
            assert steps < 64
            index = -value - 1
            if index == accept: break
            pop, nt = reductions[index]
            if pop: del stack[-pop:]
            if len(stack) == capacity:
                grown = max(8, capacity * 2)
                max_peak = max(max_peak, capacity + grown)
                capacity = grown
            stack.append(goto(stack[-1], nt))
            max_capacity = max(max_capacity, capacity)
        max_steps = max(max_steps, steps)
        if value > 0:
            expected += token < terminals
            stack.append(value - 1)
            state = tuple(stack)
            if state not in seen:
                seen.add(state)
                pending.append(state)
    max_expected = max(max_expected, expected)
print(dict(states=len(eof), terminals=terminals, reachable_shift_stacks=len(seen),
    max_live_states=max_states, max_diagnostic_states=max_diagnostic,
    max_consecutive_reductions=max_steps, max_expected_terminals=max_expected,
    max_diagnostic_capacity=max_capacity, max_diagnostic_growth_peak=max_peak))
