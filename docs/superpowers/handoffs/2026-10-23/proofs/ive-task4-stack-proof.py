from pathlib import Path
from collections import deque
import re, hashlib, json
root = Path('/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports')
source = (root/'engine/src/plantuml/generated/sequence.rs').read_text()
def table(name):
    raw = re.search(r'const '+name+r':.*?= &\[(.*?)\];', source, re.S).group(1)
    return [int(x) for x in re.findall(r'-?\d+', re.sub(r'//[^\n]*','',raw))]
action,eof=table('__ACTION'),table('__EOF_ACTION')
terminals=len(action)//len(eof)
body=source[source.index('fn __simulate_reduce'):source.index('pub struct')]
reductions={int(i):(int(p),int(n)) for i,p,n in re.findall(r'(\d+) => \{\s*__state_machine::SimulatedReduce::Reduce \{\s*states_to_pop: (\d+),\s*nonterminal_produced: (\d+)',body)}
accept=int(re.search(r'(\d+) => __state_machine::SimulatedReduce::Accept',body).group(1))
def goto(state,nt):
    if nt==6:return 6 if state==2 else 1
    if nt==9:return {7:11,4:33,5:37,10:46,14:57,15:58,16:59}.get(state,3)
    return {2:7,3:36,4:2,5:19,7:49,8:12,10:20,11:41,12:56,13:62,14:13,15:18,16:17}.get(nt,0)
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
                steps+=1; assert steps<16
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
    assert maximum<=2*depth+12
    return dict(depth=depth,reachable=len(seen),max_live=maximum,max_reductions=max_reductions,max_expected=max_expected,max_diagnostic_capacity=max_capacity,max_diagnostic_growth_peak=max_peak)
print(json.dumps(dict(sha256=hashlib.sha256(source.encode()).hexdigest(),states=len(eof),terminals=terminals,proof=[explore(depth) for depth in [0,1,2,8,64,65]]),indent=2))
