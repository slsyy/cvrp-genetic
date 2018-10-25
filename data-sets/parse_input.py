#!/usr/bin/env python3

import sys
import json
import re

from enum import Enum

class Section(Enum):
    NONE = 1,
    NODE = 2,
    DEMAND = 3,
    DEPOT = 4,


res = {}
res["nodes"] = {}

section = Section.NONE

for line in sys.stdin.readlines():
    def extract(regex):
        return re.findall(regex, line)[0]

    def match(regex):
        return re.search(regex, line)

    if match(r"NODE_COORD_SECTION"):
        section = Section.NODE
        continue

    if match(r"DEMAND_SECTION"):
        section = Section.DEMAND
        continue

    if match(r"DEPOT_SECTION"):
        section = Section.DEPOT
        continue


    if section == Section.NODE:
        n, x, y = line.split()
        res["nodes"][n] = {"x": float(x), "y": float(y), "isDepot": False}

    if section == Section.DEMAND:
        n, d = line.split()
        res["nodes"][n].update(demand=int(d))

    if section == Section.DEPOT:
        n = int(line)
        if n == -1:
            break

        res["nodes"][str(n)].update(isDepot=True)

    if match(r"CAPACITY"):
        res["capacity"] = int(extract(r"\d+"))
    elif match(r"EDGE_WEIGHT_TYPE"):
        res["edgeWeightType"] = extract(r"\w+$")


print(json.dumps(res, indent=4))