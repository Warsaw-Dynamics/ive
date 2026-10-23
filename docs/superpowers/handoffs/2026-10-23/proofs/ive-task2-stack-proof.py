from pathlib import Path
from collections import deque
import re

root = Path('/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports')
for name in ['document', 'header']:
    source = (root / f'engine/src/plantuml/generated/{name}.rs').read_text()
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
        if name == 'header':
            return {0: 1, 1: 3, 2: 29, 3: 2}.get(nt, 0)
        if nt == 2:
            return {28: 49, 29: 50, 30: 51, 31: 52}.get(state, 32)
        if nt == 3:
            return state + 13 if 2 <= state <= 13 else 14
        return {4: 33, 5: 35, 6: 48}.get(nt, 0)
    pending, seen = deque([(0,)]), {(0,)}
    max_states = max_diagnostic = max_steps = max_expected = max_diagnostic_capacity = max_diagnostic_peak = 0
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
                assert len(stack) <= 8
                value = eof[stack[-1]] if token == terminals else action[stack[-1] * terminals + token]
                if value >= 0:
                    break
                steps += 1
                assert steps <= 8
                index = -value - 1
                if index == accept:
                    break
                pop, nt = reductions[index]
                if pop:
                    del stack[-pop:]
                if len(stack) == capacity:
                    grown = max(8, capacity * 2)
                    max_diagnostic_peak = max(max_diagnostic_peak, capacity + grown)
                    capacity = grown
                stack.append(goto(stack[-1], nt))
                max_diagnostic_capacity = max(max_diagnostic_capacity, capacity)
            max_steps = max(max_steps, steps)
            if value > 0:
                expected += token < terminals
                stack.append(value - 1)
                state = tuple(stack)
                if state not in seen:
                    seen.add(state)
                    pending.append(state)
        max_expected = max(max_expected, expected)
    print(name, dict(states=len(eof), terminals=terminals, reachable_shift_stacks=len(seen),
        max_live_states=max_states, max_diagnostic_states=max_diagnostic,
        max_consecutive_reductions=max_steps, max_expected_terminals=max_expected,
        max_diagnostic_capacity=max_diagnostic_capacity, max_diagnostic_growth_peak=max_diagnostic_peak))
