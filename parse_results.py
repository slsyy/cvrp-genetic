#!/usr/bin/env python3

import sys
import json

import matplotlib.pyplot as plt
import networkx as nx

j = json.load(sys.stdin)

nodes = j["description"]["nodes"]
best_path = j["bestPath"]
best_solution_performance = j["bestSolutionPerformance"]

G = nx.DiGraph()


max_size = max([node["demand"]for (_, node) in nodes.items()])

for id, node in nodes.items():
    attrs = {
        "pos": (node["x"], node["y"]),
        "color": "#F50057" if node["isDepot"] else "#76FF03",
        "size": 1000 * node["demand"] / max_size or 600
    }
    G.add_node(id, **attrs)

edge_colors = sorted([
    "#E65100",
    "#1B5E20",
    "#1A237E",
    "#4A148C",
    "#827717",
    "#3E2723",
    "#B71C1C",
    "#0D47A1",
], key=hash)

for i, path in enumerate(best_path):
    for x, y in zip(path, path[1:]):
        G.add_edge(x, y, color=edge_colors[i])
    

nodes = G.nodes()
edges = G.edges()

pos=nx.get_node_attributes(G, 'pos')
# pos=nx.spring_layout(G, pos=nx.get_node_attributes(G, 'pos'), iterations=10000)

plt.style.use('Solarize_Light2')

plt.subplot(121)
plt.xlabel("X")
plt.ylabel("Y")
nx.draw_networkx(
    G, 
    alpha=0.5,
    pos=pos,
    node_color=[G.node[n]["color"] for n in nodes],
    node_size=[G.node[n]["size"] for n in nodes],
    edge_color=[G[x][y]["color"] for (x, y)in edges]
)

plt.subplot(122)
x = [x[0] for x in best_solution_performance]
y = [x[1] for x in best_solution_performance]
plt.scatter(x, y, c="#E91E63")
plt.xlabel("Generation")

xticks = [i for i in list(plt.xticks()[0]) if i < x[-1]][:-1]  + [x[-1]]
yticks = [i for i in list(plt.yticks()[0]) if i > y[-1]][1:]  + [y[-1]]

plt.xticks(xticks)
plt.yticks(yticks)
plt.xlim(left=-0.05*max(x))

plt.ylabel("Cost")

plt.show()