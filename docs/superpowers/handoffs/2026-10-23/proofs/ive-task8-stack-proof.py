from pathlib import Path
from collections import deque, defaultdict
import re, hashlib, json, sys
sys.setrecursionlimit(10000)

root = Path('/mnt/data/Projects/ive/.worktree/diagram-activity-parallel/diagram-ports')
source = (root/'engine/src/plantuml/generated/activity.rs').read_text()

def table(name):
    raw = re.search(r'const '+name+r':.*?= &\[(.*?)\];', source, re.S).group(1)
    return [int(x) for x in re.findall(r'-?\d+', re.sub(r'//[^\n]*','',raw))]

action,eof=table('__ACTION'),table('__EOF_ACTION')
terminals=len(action)//len(eof)
body=source[source.index('fn __simulate_reduce'):source.index('pub struct')]
reductions={int(i):(int(p),int(n)) for i,p,n in re.findall(r'(\d+) => \{\s*__state_machine::SimulatedReduce::Reduce \{\s*states_to_pop: (\d+),\s*nonterminal_produced: (\d+)',body)}
accept=int(re.search(r'(\d+) => __state_machine::SimulatedReduce::Accept',body).group(1))

def goto(state,nt):
    nt-=2
    if nt==1:return {5:9,10:17,12:19,13:20,16:22,26:30,33:36,34:37}.get(state,1)
    if nt==5:return {14:67,25:78,27:83,35:91}.get(state,62)
    if nt==6:return 84 if state==28 else 49
    if nt==14:return {2:6,28:6,4:8,24:29,32:90}.get(state,63)
    if nt==15:return 71 if state==18 else 58
    return {0:38,2:75,3:23,4:50,7:39,8:60,9:21,10:72,11:15,12:54,13:89,16:77,17:31,18:57}.get(nt,0)

open_tokens={3,7,9,11,15,19}
open_states={value-1 for state in range(len(eof)) for token in open_tokens if (value:=action[state*terminals+token])>0}
edges=set()

def explore(depth):
    pending,seen=deque([(0,)]),{(0,)}
    maximum=max_reductions=max_expected=max_capacity=max_peak=0
    states=set()
    while pending:
        original=pending.popleft()
        maximum=max(maximum,len(original))
        expected=0
        for token in range(terminals+1):
            stack=list(original); capacity=len(stack); steps=0
            while True:
                states.add(stack[-1])
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
                target=goto(stack[-1],nt)
                edges.add((stack[-1],target,0))
                stack.append(target);max_capacity=max(max_capacity,capacity)
                if sum(s in open_states for s in stack)>depth:break
            max_reductions=max(max_reductions,steps)
            if value>0:
                expected+=token<terminals
                target=value-1
                edges.add((stack[-1],target,int(token in open_tokens)))
                stack.append(target);state=tuple(stack)
                if sum(s in open_states for s in stack)<=depth and state not in seen:seen.add(state);pending.append(state)
        max_expected=max(max_expected,expected)
    return dict(depth=depth,reachable=len(seen),states=len(states),edges=len(edges),max_live=maximum,max_reductions=max_reductions,max_expected=max_expected,max_diagnostic_capacity=max_capacity,max_diagnostic_growth_peak=max_peak)

proof=[explore(depth) for depth in [0,1,2,3,4]]

# Every generated shift edge is part of the viable-prefix graph. Goto pairs
# are enumerated by all reductions/terminals/EOF at every reachable LR state.
for state in range(len(eof)):
    for token in range(terminals):
        value=action[state*terminals+token]
        if value>0:edges.add((state,value-1,int(token in open_tokens)))

graph=defaultdict(list)
for left,right,cost in edges:graph[left].append((right,cost))

closure={(s,v-1,int(t in open_tokens)) for s in range(len(eof)) for t in range(terminals) if (v:=action[s*terminals+t])>0}
while True:
    reverse=defaultdict(set)
    for left,right,cost in closure:reverse[right].add(left)
    additions=set()
    for state in range(len(eof)):
        for value in set(action[state*terminals:(state+1)*terminals]+[eof[state]]):
            if value>=0 or -value-1==accept:continue
            pop,nt=reductions[-value-1]
            previous={state}
            for _ in range(pop):previous={left for right in previous for left in reverse[right]}
            for left in previous:additions.add((left,goto(left,nt),0))
    if additions<=closure:break
    closure|=additions
assert closure==edges, ('finite transition closure mismatch',len(closure),len(edges),sorted(closure-edges))

def longest(depth):
    memo={};active=set()
    def visit(state,left):
        key=state,left
        if key in memo:return memo[key]
        assert key not in active, ('zero-cost recursive stack cycle',key)
        active.add(key)
        value=1+max((visit(target,left-cost) for target,cost in graph[state] if cost<=left),default=0)
        active.remove(key);memo[key]=value
        return value
    return visit(0,depth)

def prefix_distances(depth):
    result={0:1}
    for used in range(depth+1):
        while True:
            updated=dict(result)
            for left,right,cost in edges:
                if cost==0 and left in result:updated[right]=max(updated.get(right,0),result[left]+1)
            if updated==result:break
            result=updated
        if used==depth:return result
        result={right:max(result.get(right,0),max((result.get(left, -100000)+1 for left,target,cost in edges if target==right and cost==1),default=-100000)) for right in range(len(eof))}
        result={state:value for state,value in result.items() if value>0}

distance3,distance4=prefix_distances(3),prefix_distances(4)
assert len(distance3)==len(distance4)==len(eof)
assert max(distance3.values())==33
assert all(distance4[state]<=distance3[state]+9 for state in distance3)

output=dict(sha256=hashlib.sha256(source.encode()).hexdigest(),states=len(eof),terminals=terminals,open_states=sorted(open_states),proof=proof,viable_prefix_edges=len(edges),reduction_closure_edges=len(closure),max_plus_induction='all 92 end-state distances reachable by depth3; D4<=D3+9 componentwise; max D3=33',weighted_path_bounds={d:longest(d) for d in [0,1,2,8,64,65,128]})
print(json.dumps(output,indent=2))
