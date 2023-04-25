import asyncio
from codecs import decode
from typing import Optional
from venv import create
import hydro
import json
from pathlib import Path
from aiostream import stream

import pandas as pd
import numpy as np
import uuid


class Tree:
    def __init__(self, node, left, right):
        self.node = node
        self.left = left
        self.right = right

    def map(self, transform):
        return Tree(
            transform(self.node),
            self.left.map(transform) if self.left is not None else None,
            self.right.map(transform) if self.right is not None else None
        )

    def flatten_with_path(self, cur_path=""):
        return [(self.node, cur_path)] + \
            (self.left.flatten_with_path(cur_path + "L") if self.left is not None else []) + \
            (self.right.flatten_with_path(cur_path + "R") if self.right is not None else [])

    async def map_async(self, transform):
        return Tree(
            await transform(self.node),
            (await self.left.map_async(transform)) if self.left is not None else None,
            (await self.right.map_async(transform)) if self.right is not None else None
        )

def create_tree(depth, deployment, create_machine) -> Optional[Tree]:
    if depth == 0:
        return None
    else:
        self_service = deployment.HydroflowCrate(
            src=str(Path(__file__).parent.absolute()),
            example="topolotree",
            on=create_machine()
        )

        left = create_tree(depth - 1, deployment, create_machine)
        right = create_tree(depth - 1, deployment, create_machine)

        if left is not None:
            self_service.ports.to_left.send_to(left.node.ports.from_parent)
            left.node.ports.to_parent.send_to(self_service.ports.from_left)
        else:
            self_service.ports.to_left.send_to(hydro.null())
            hydro.null().send_to(self_service.ports.from_left)

        if right is not None:
            self_service.ports.to_right.send_to(right.node.ports.from_parent)
            right.node.ports.to_parent.send_to(self_service.ports.from_right)
        else:
            self_service.ports.to_right.send_to(hydro.null())
            hydro.null().send_to(self_service.ports.from_right)

        return Tree(
            self_service,
            left,
            right
        )

async def run_experiment(deployment, machine_pool, experiment_id, summaries_file, tree_arg, depth_arg, clients_arg, is_gcp, gcp_vpc):
    tree_depth = int(depth_arg)
    is_tree = tree_arg == "topolo" # or "pn"

    num_replicas = 2 ** tree_depth - 1

    num_clients = int(clients_arg)

    localhost_machine = deployment.Localhost()

    currently_deployed = []
    def create_machine():
        if len(machine_pool) > 0:
            print("Using machine from pool")
            ret = machine_pool.pop()
            currently_deployed.append(ret)
            return ret
        else:
            if is_gcp:
                out = deployment.GCPComputeEngineHost(
                    project="hydro-chrisdouglas",
                    machine_type="n2-standard-4",
                    image="debian-cloud/debian-11",
                    region="us-west1-a",
                    network=gcp_vpc
                )
            else:
                out = localhost_machine
            currently_deployed.append(out)
            return out

    all_nodes = []
    if is_tree:
        tree = create_tree(tree_depth, deployment, create_machine)
        tree.node.ports.to_parent.send_to(hydro.null())
        hydro.null().send_to(tree.node.ports.from_parent)
        all_nodes = [tup[0] for tup in tree.flatten_with_path()]
    else:
        cluster = [
            deployment.HydroflowCrate(
                src=str(Path(__file__).parent.absolute()),
                example="pn_counter" if tree_arg == "pn" else "pn_counter_delta",
                args=[json.dumps([i]), json.dumps([num_replicas])],
                on=create_machine()
            )
            for i in range(num_replicas)
        ]

        for i in range(num_replicas):
            cluster[i].ports.to_peer.send_to(hydro.demux(
                {
                    j: cluster[j].ports.from_peer.merge()
                    for j in range(num_replicas)
                    if i != j
                }
            ))

        all_nodes = cluster

    if is_tree:
        source = tree
        while source.left is not None:
            source = source.left
        source = source.node

        dest = tree
        while dest.right is not None:
            dest = dest.right
        dest = dest.node
    else:
        source = cluster[0]
        dest = cluster[-1]
        
    for node in all_nodes:
        if node is not dest:
            node.ports.query_responses.send_to(hydro.null())
        if node is not source:
            hydro.null().send_to(node.ports.increment_requests)

    latency_measurer = deployment.HydroflowCrate(
        src=str(Path(__file__).parent.absolute()),
        example="topolotree_latency_measure",
        args=[json.dumps([num_clients])],
        on=create_machine()
    )

    latency_measurer.ports.increment_start_node.send_to(source.ports.increment_requests.merge())
    dest.ports.query_responses.send_to(latency_measurer.ports.end_node_query)

    await deployment.deploy()

    print("deployed!")

    latency = []
    memory_per_node = [[] for _ in range(num_replicas)]
    throughput_raw = []

    throughput = []

    latency_stdout = await latency_measurer.stdout()

    memories_streams_with_index = [
        stream.map(
            await node.stdout(),
            lambda x,i=i: (i, x)
        )
        for i, node in enumerate(all_nodes)
    ]

    async def memory_plotter():
        try:
            async with stream.merge(*memories_streams_with_index).stream() as merged:
                async for node_idx, line in merged:
                    line_split = line.split(",")
                    if line_split[0] == "memory":
                        memory_per_node[node_idx].append(int(line_split[1]))
        except asyncio.CancelledError:
            return

    memory_plotter_task = asyncio.create_task(memory_plotter())

    async def latency_plotter():
        try:
            async for line in latency_stdout:
                line_split = line.split(",")
                if line_split[0] == "throughput":
                    count = int(line_split[1])
                    period = float(line_split[2])
                    throughput_raw.append([count, period])
                    throughput.append(count / period)
                elif line_split[0] == "latency":
                    number = int(line_split[1]) # microseconds
                    latency.append(number)
        except asyncio.CancelledError:
            return

    latency_plotter_task = asyncio.create_task(latency_plotter())

    await deployment.start()
    print("started!")

    await asyncio.sleep(30)

    await latency_measurer.stop()
    await asyncio.gather(*[node.stop() for node in all_nodes])

    memory_plotter_task.cancel()
    await memory_plotter_task

    latency_plotter_task.cancel()
    await latency_plotter_task

    def summarize(v, kind):
        print("mean = ", np.mean(v))
        print("std = ", np.std(v))
        print("min = ", np.min(v))
        print("max = ", np.max(v))
        print("percentile 99 = ", np.percentile(v, 99))
        print("percentile 75 = ", np.percentile(v, 75))
        print("percentile 50 = ", np.percentile(v, 50))
        print("percentile 25 = ", np.percentile(v, 25))
        print("percentile 1 = ", np.percentile(v, 1))
        
        summaries_file.write("\n")
        summaries_file.write(tree_arg + ",")
        summaries_file.write(str(tree_depth) + ",")
        summaries_file.write(str(num_clients) + ",")
        summaries_file.write(kind + ",")
        summaries_file.write(str(np.mean(v)) + ",")
        summaries_file.write(str(np.std(v)) + ",")
        summaries_file.write(str(np.min(v)) + ",")
        summaries_file.write(str(np.max(v)) + ",")
        summaries_file.write(str(np.percentile(v, 99)) + ",")
        summaries_file.write(str(np.percentile(v, 75)) + ",")
        summaries_file.write(str(np.percentile(v, 50)) + ",")
        summaries_file.write(str(np.percentile(v, 25)) + ",")
        summaries_file.write(str(np.percentile(v, 1)))
        summaries_file.flush()

    print("latency:")
    summarize(latency, "latency")

    print("throughput:")
    summarize(throughput, "throughput")

    init_memory = [
        memory[0]
        for memory in memory_per_node
    ]
    print("init memory:")
    summarize(init_memory, "init_memory")

    final_memory = [
        memory[-1]
        for memory in memory_per_node
    ]
    print("final memory:")
    summarize(final_memory, "final_memory")

    pd.DataFrame(latency).to_csv("latency_" + tree_arg + "_tree_depth_" + str(tree_depth) + "_num_clients_" + str(num_clients) + "_" + experiment_id+".csv", index=False, header=["latency"])
    pd.DataFrame(throughput_raw).to_csv("throughput_" + tree_arg + "_tree_depth_" + str(tree_depth) + "_num_clients_" + str(num_clients) + "_" + experiment_id+".csv", index=False, header=["count", "period"])
    pd.DataFrame(init_memory).to_csv("init_memory_" + tree_arg + "_tree_depth_" + str(tree_depth) + "_num_clients_" + str(num_clients) + "_" + experiment_id+".csv", index=False, header=["memory"])
    pd.DataFrame(final_memory).to_csv("final_memory_" + tree_arg + "_tree_depth_" + str(tree_depth) + "_num_clients_" + str(num_clients) + "_" + experiment_id+".csv", index=False, header=["memory"])

    for machine in currently_deployed:
        machine_pool.append(machine)

# rustup run nightly-2023-04-13-x86_64-unknown-linux-gnu hydro deploy ../hydro_cli_examples/toplotree_latency.hydro.py -- local/gcp DEPTH_OF_TREE
async def main(args):
    # the current timestamp
    import datetime
    experiment_id = str(datetime.datetime.now())

    summaries_file = open(f"summaries_{experiment_id}.csv", "w")
    summaries_file.write("protocol,tree_depth,num_clients,kind,mean,std,min,max,percentile_99,percentile_75,percentile_50,percentile_25,percentile_1")

    deployment = hydro.Deployment()
    pool = []

    network = hydro.GCPNetwork(
        project="hydro-chrisdouglas",
    ) if args[0] == "gcp" else None

    for depth_arg in args[2].split(","):
        for tree_arg in args[1].split(","):
            for num_clients_arg in args[3].split(","):
                await run_experiment(deployment, pool, experiment_id, summaries_file, tree_arg, depth_arg, num_clients_arg, args[0] == "gcp", network)
            
    summaries_file.close()

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
