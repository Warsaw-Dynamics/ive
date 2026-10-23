from pathlib import Path
from collections import deque
import hashlib, json, re

ROOT=Path('/mnt/data/Projects/ive/.worktree/diagram-data-parallel/diagram-ports')
EDGES={}

def json_goto(state, nt):
    if nt==2:return {7:10,11:16}.get(state,9)
    if nt==5:return {5:36,8:40,12:44}.get(state,27)
    if nt==7:return 12 if state==8 else 5
    if nt==9:return 8 if state==2 else 37 if state in (6,7,11) else 43 if state in (9,10,16) else 1
    if nt==12:return 47 if state==15 else 32
    if nt==20:return {3:30,13:45,14:46}.get(state,19)
    return {3:28,4:29,10:17,11:31,13:33,14:34,15:2,17:18,18:11}.get(nt,0)

def hcl_goto(state, nt):
    if nt==2:return 38 if state==12 else 32
    if nt==3:return 21 if state==3 else 15
    if nt==9:return {9:11,10:13}.get(state,5)
    if nt==12:return 23 if state==4 else 1
    if nt==13:return 35 if state in (8,12) else 16
    if nt==16:return {1:17,3:17,4:24,7:31}.get(state,6)
    return {4:2,5:3,6:33,7:34,8:25,10:14,11:4,14:7,15:30}.get(nt,0)

def load(family):
    source=(ROOT/f'engine/src/plantuml/generated/{family}.rs').read_text()
    def table(name):
        raw=re.search(r'const '+name+r':.*?= &\[(.*?)\];',source,re.S).group(1)
        return [int(x) for x in re.findall(r'-?\d+',re.sub(r'//[^\n]*','',raw))]
    action,eof=table('__ACTION'),table('__EOF_ACTION')
    body=source[source.index('fn __simulate_reduce'):source.index('pub struct')]
    reductions={int(i):(int(p),int(n)) for i,p,n in re.findall(r'(\d+) => \{\s*__state_machine::SimulatedReduce::Reduce \{\s*states_to_pop: (\d+),\s*nonterminal_produced: (\d+)',body)}
    accept=int(re.search(r'(\d+) => __state_machine::SimulatedReduce::Accept',body).group(1))
    return source,action,eof,reductions,accept

def explore(family, depth):
    source,action,eof,reductions,accept=load(family)
    goto=json_goto if family=='json' else hcl_goto
    terminals=len(action)//len(eof)
    pending=deque([((0,),0)])
    seen={((0,),0)}
    maximum=consecutive=expected_max=capacity_max=peak=0
    longest=[]
    edges=set()
    while pending:
        original,level=pending.popleft()
        if len(original)>maximum:maximum=len(original);longest=original
        expected=0
        for token in range(terminals+1):
            stack=list(original);capacity=len(stack);steps=0
            while True:
                if len(stack)>maximum:maximum=len(stack);longest=tuple(stack)
                value=eof[stack[-1]] if token==terminals else action[stack[-1]*terminals+token]
                if value>=0:break
                steps+=1;assert steps<64
                reduction=-value-1
                if reduction==accept:break
                pop,nt=reductions[reduction]
                if pop:del stack[-pop:]
                if len(stack)==capacity:
                    grown=max(8,capacity*2);peak=max(peak,capacity+grown);capacity=grown
                target=goto(stack[-1],nt)
                edges.add((stack[-1],target,int(family=='hcl' and nt==4)))
                stack.append(target);capacity_max=max(capacity_max,capacity)
            consecutive=max(consecutive,steps)
            if value>0:
                expected+=token<terminals
                next_level=level+int(token in (0,2))-int(token in (1,3))
                if next_level>depth:continue
                assert next_level>=0
                stack.append(value-1);state=(tuple(stack),next_level)
                edges.add((original[-1] if steps==0 else stack[-2],value-1,int(token in (0,2))-int(token in (1,3))))
                if state not in seen:seen.add(state);pending.append(state)
        expected_max=max(expected_max,expected)
    EDGES[(family,depth)]=edges
    return dict(depth=depth,reachable=len(seen),max_live=maximum,max_reductions=consecutive,max_expected=expected_max,max_diagnostic_capacity=capacity_max,max_diagnostic_growth_peak=peak,longest=list(longest))

def graph_bound(family,depth):
    edges=EDGES[(family,4)]
    assert edges==EDGES[(family,3)]
    outgoing={}
    for start,end,delta in edges:outgoing.setdefault(start,[]).append((end,delta))
    initial=(0,0)
    pending=deque([initial]);seen={initial};graph={};indegree={initial:0}
    while pending:
        node=pending.popleft();state,level=node;graph[node]=[]
        for end,delta in outgoing.get(state,[]):
            target=(end,level+delta)
            if not 0<=target[1]<=depth:continue
            graph[node].append(target);indegree[target]=indegree.get(target,0)+1
            if target not in seen:seen.add(target);pending.append(target)
    pending=deque([n for n in seen if indegree[n]==0]);distance={initial:1};visited=0
    while pending:
        node=pending.popleft();visited+=1
        for target in graph[node]:
            distance[target]=max(distance.get(target,0),distance.get(node,0)+1)
            indegree[target]-=1
            if indegree[target]==0:pending.append(target)
    assert visited==len(seen),(family,depth,'nonpositive-cost viable-prefix cycle')
    return dict(depth=depth,graph_nodes=len(seen),max_live=max(distance.values()))

if __name__=='__main__':
    output={family:{'sha256':hashlib.sha256(load(family)[0].encode()).hexdigest(),'states':len(load(family)[2]),'terminals':len(load(family)[1])//len(load(family)[2]),'exhaustive':[explore(family,d) for d in (0,1,2,3,4)]} for family in ('json','hcl')}
    for family in output:output[family]['graph_proof']=[graph_bound(family,d) for d in (0,1,2,8,64,65,128)]
    print(json.dumps(output,indent=2))
