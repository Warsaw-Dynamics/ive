from pathlib import Path
from collections import deque
import re, hashlib, json

root = Path('/mnt/data/Projects/ive/.worktree/diagram-state-parallel/diagram-ports')
source = (root/'engine/src/plantuml/generated/state.rs').read_text()

def table(name):
    raw = re.search(r'const '+name+r':.*?= &\[(.*?)\];', source, re.S).group(1)
    return [int(x) for x in re.findall(r'-?\d+', re.sub(r'//[^\n]*','',raw))]

action,eof=table('__ACTION'),table('__EOF_ACTION')
terminals=len(action)//len(eof)
body=source[source.index('fn __simulate_reduce'):source.index('pub struct')]
reductions={int(i):(int(p),int(n)) for i,p,n in re.findall(r'(\d+) => \{\s*__state_machine::SimulatedReduce::Reduce \{\s*states_to_pop: (\d+),\s*nonterminal_produced: (\d+)',body)}
accept=int(re.search(r'(\d+) => __state_machine::SimulatedReduce::Accept',body).group(1))

def goto(state,nt):
    if nt==6:return 47 if state==11 else 26
    if nt==9:return 52 if state==12 else 44
    if nt==10:return 38 if state==7 else 24
    if nt==12:return 5 if state==2 else 1
    if nt==16:return {8:40,13:58}.get(state,33)
    if nt==18:return 16 if state in (1,5) else 34
    return {2:6,3:29,4:2,5:11,7:48,8:9,11:15,13:42,14:3,15:4,17:49,19:14}.get(nt,0)

def explore(depth):
    pending,seen=deque([(0,)]),{(0,)}
    maximum=max_reductions=max_expected=max_capacity=max_peak=0
    while pending:
        original=pending.popleft()
        maximum=max(maximum,len(original))
        expected=0
        for token in range(terminals+1):
            stack=list(original); capacity=len(stack); steps=0
            while True:
                maximum=max(maximum,len(stack))
                value=eof[stack[-1]] if token==terminals else action[stack[-1]*terminals+token]
                if value>=0:break
                steps+=1; assert steps<32
                reduction=-value-1
                if reduction==accept:break
                pop,nt=reductions[reduction]
                if pop:del stack[-pop:]
                if len(stack)==capacity:
                    grown=max(8,capacity*2);max_peak=max(max_peak,capacity+grown);capacity=grown
                stack.append(goto(stack[-1],nt));max_capacity=max(max_capacity,capacity)
                if stack.count(2)>depth:break
            max_reductions=max(max_reductions,steps)
            if value>0:
                expected+=token<terminals
                stack.append(value-1);state=tuple(stack)
                if stack.count(2)<=depth and state not in seen:seen.add(state);pending.append(state)
        max_expected=max(max_expected,expected)
    assert maximum<=2*depth+16
    return dict(depth=depth,reachable=len(seen),max_live=maximum,max_reductions=max_reductions,max_expected=max_expected,max_diagnostic_capacity=max_capacity,max_diagnostic_growth_peak=max_peak)

print(json.dumps(dict(sha256=hashlib.sha256(source.encode()).hexdigest(),states=len(eof),terminals=terminals,proof=[explore(depth) for depth in [0,1,2,8,64,65]]),indent=2))
