#!/bin/bash
# Oracle solutions for skills-bench tasks.
# Writes the correct output files so the grader scores 1.0.
# Runs as root with /root and /output writable; solution.sh is mounted read-only
# at /oracle-solution.sh and never baked into the agent image.
set -euo pipefail

TASK_ID="${EVAL_TASK_ID:-citation-check}"

case "$TASK_ID" in
  citation-check)
    cat > /root/answer.json <<'JSON'
{
  "fake_citations": [
    "Advances in Artificial Intelligence for Natural Language Processing",
    "Blockchain Applications in Supply Chain Management",
    "Neural Networks in Deep Learning: A Comprehensive Review"
  ]
}
JSON
    ;;

  bike-rebalance)
    # Run the upstream SCIP reference solver (requires pyscipopt, already installed).
    python3 << 'PY'
from __future__ import annotations
import json, math, time
from pathlib import Path
from typing import Any
from pyscipopt import Model, quicksum

START = "depot_start"; END = "depot_end"; EPS = 1e-7
DATA_PATH = Path("/root/data.json"); OUTPUT_PATH = Path("/root/report.json")
TIME_LIMIT_SECONDS = 300.0; SCIP_SEED = 0; SCIP_THREADS = 1

def log(m): print(f"[bike-rebalance] {m}", flush=True)
def safe_stat(model, name):
    m = getattr(model, name, None)
    try: return str(m()) if m else "?"
    except: return "?"
def set_param(model, name, val):
    try: model.setParam(name, val); return True
    except: return False
def great_circle(a, b):
    r = math.pi/180
    p1=(90-a["latitude"])*r; p2=(90-b["latitude"])*r
    t1=a["longitude"]*r; t2=b["longitude"]*r
    c=math.sin(p1)*math.sin(p2)*math.cos(t1-t2)+math.cos(p1)*math.cos(p2)
    return math.acos(max(-1.,min(1.,c)))*3960.
def clean(v,d=6):
    v=float(v)
    if abs(v)<EPS: return 0.
    r=round(v,d)
    return float(round(r)) if abs(r-round(r))<EPS else r
def loc(node, depot, stations):
    return depot if node in (START,END) else stations[int(node)]

data = json.loads(DATA_PATH.read_text())
stations = data["stations"]; depot = data["depot"]
V = int(data["vehicle_count"]); C = int(data["vehicle_capacity"]); W = float(data["penalty_weight"])
S = list(range(len(stations))); Vs = list(range(V))
from_nodes=[START,*S]; to_nodes=[*S,END]
arcs=[(i,j) for i in from_nodes for j in to_nodes if i!=j]
dist={(i,j):great_circle(loc(i,depot,stations),loc(j,depot,stations)) for i,j in arcs}

m=Model("bike_rebalance"); m.hideOutput()
for name,val in [("randomization/randomseedshift",SCIP_SEED),("randomization/permutationseed",SCIP_SEED),
                 ("randomization/lpseed",SCIP_SEED),("parallel/maxnthreads",SCIP_THREADS)]:
    set_param(m,name,val)

x={(v,i,j):m.addVar(vtype="B") for v in Vs for i,j in arcs}
ld={(v,i):m.addVar(vtype="I",lb=0,ub=C) for v in Vs for i in [START,END,*S]}
sv={(v,i):m.addVar(vtype="I",lb=-C,ub=C) for v in Vs for i in S}
od={(v,i):m.addVar(vtype="C",lb=1,ub=max(1,len(S))) for v in Vs for i in S}
un={i:m.addVar(vtype="I",lb=0) for i in S}

for v in Vs:
    m.addCons(quicksum(x[v,START,j] for j in S)==1)
    m.addCons(quicksum(x[v,i,END] for i in S)==1)
    for i in S:
        inc=quicksum(x[v,j,i] for j in from_nodes if j!=i)
        out=quicksum(x[v,i,j] for j in to_nodes if j!=i)
        m.addCons(inc==out); m.addCons(out<=1)
        m.addCons(sv[v,i]<=C*out); m.addCons(sv[v,i]>=-C*out)
    for i,j in arcs:
        op=sv[v,j] if isinstance(j,int) else 0
        m.addCons(ld[v,j]-ld[v,i]-op<=2*C*(1-x[v,i,j]))
        m.addCons(ld[v,j]-ld[v,i]-op>=-2*C*(1-x[v,i,j]))
    for i in S:
        for j in S:
            if i!=j: m.addCons(od[v,i]-od[v,j]+len(S)*x[v,i,j]<=len(S)-1)

for i,s in enumerate(stations):
    ini=int(s["initial_bikes"]); space=max(0,int(s["station_capacity"])-ini)
    nc=quicksum(sv[v,i] for v in Vs); req=int(s["net_rebalancing_target"])
    m.addCons(nc<=ini); m.addCons(nc>=-space)
    m.addCons(nc-req<=un[i]); m.addCons(req-nc<=un[i])

m.setObjective(quicksum(dist[i,j]*x[v,i,j] for v in Vs for i,j in arcs)+W*quicksum(un[i] for i in S),"minimize")
m.setParam("limits/time",TIME_LIMIT_SECONDS)
log("solving..."); m.optimize()
if m.getNSols()==0: raise RuntimeError("no solution found")

def route(v):
    out={i:j for i,j in arcs if m.getVal(x[v,i,j])>0.5}
    r=[START]; cur=START; seen={START}
    while cur!=END:
        cur=out[cur]
        if cur in seen and cur!=END: raise RuntimeError("cycle")
        r.append(cur); seen.add(cur)
    return r

def sid(node): return int(stations[node]["id"])

vreps=[]; tdist=0.; pp=dict.fromkeys(S,0.); pd=dict.fromkeys(S,0.)
for v in Vs:
    rn=route(v)
    stops=[]
    for a,b in zip(rn,rn[1:]):
        tdist+=dist[a,b]
        if isinstance(b,int):
            sv_=m.getVal(sv[v,b]); pk=clean(max(sv_,0)); dr=clean(max(-sv_,0))
            pp[b]+=pk; pd[b]+=dr
            stops.append({"station_id":sid(b),"bikes_picked_up":pk,"bikes_dropped_off":dr,"load_after_stop":clean(m.getVal(ld[v,b]))})
    vreps.append({"vehicle_id":v+1,"start_load":clean(m.getVal(ld[v,START])),"route":[x if isinstance(x,str) else sid(x) for x in rn],"stops":stops,"end_load":clean(m.getVal(ld[v,END]))})

sreps=[]; tunmet=0.
for i,s in enumerate(stations):
    pk=clean(pp[i]); dr=clean(pd[i]); nc=clean(pk-dr); req=float(s["net_rebalancing_target"]); um=clean(abs(req-nc))
    tunmet+=um
    sreps.append({"station_id":int(s["id"]),"net_rebalancing_target":clean(req),"total_bikes_picked_up":pk,"total_bikes_dropped_off":dr,"net_bike_change":nc,"unmet_rebalancing_amount":um})

pen=W*tunmet
OUTPUT_PATH.write_text(json.dumps({"summary":{"objective":clean(tdist+pen),"travel_distance_miles":clean(tdist),"unmet_rebalancing_penalty":clean(pen),"total_unmet_rebalancing_amount":clean(tunmet)},"vehicles":vreps,"stations":sreps},indent=2)+"\n")
log(f"wrote {OUTPUT_PATH}")
PY
    ;;

  civ6-adjacency-optimizer)
    # Use the ground truth optimal placement for scenario_3.
    # Ground truths are root-only (/tests/ground_truths); oracle runs as root.
    python3 << 'PY'
import json
from pathlib import Path

gt_path = Path("/tests/ground_truths/scenario_3/ground_truth.json")
output_path = Path("/output/scenario_3.json")
output_path.parent.mkdir(parents=True, exist_ok=True)

gt = json.loads(gt_path.read_text())
ref = gt.get("reference_solution", {})
solution = {
    "city_center": ref.get("city_center"),
    "placements": ref.get("placements", {}),
    "adjacency_bonuses": ref.get("adjacency_bonuses", {}),
    "total_adjacency": gt.get("optimal_adjacency", 0),
}
output_path.write_text(json.dumps(solution, indent=2))
print(f"wrote {output_path} (total_adjacency: {solution['total_adjacency']})")
PY
    ;;

  *)
    echo "No oracle solution for task: $TASK_ID" >&2
    exit 1
    ;;
esac
