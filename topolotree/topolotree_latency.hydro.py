import asyncio
from codecs import decode
from typing import List, Optional
import hydro
import json
from pathlib import Path
from aiostream import stream

import pandas as pd
import numpy as np
import uuid


# given a list of IDs for each node in a binary tree,
# organize the nodes into a binary tree and compute the neighbors
# of each node
def get_neighbors_in_binary_tree(flat: List[int]) -> List[List[int]]:
    tree = []
    for i in range(len(flat)):
        tree.append([])
        if i > 0:
            # add the parent
            tree[i].append((i - 1) // 2)
        if 2 * i + 1 < len(flat):
            # add the left child
            tree[i].append(2 * i + 1)
        if 2 * i + 2 < len(flat):
            # add the right child
            tree[i].append(2 * i + 2)
    return tree

def get_leaves_in_binary_tree(flat: List[int]) -> List[int]:
    tree = get_neighbors_in_binary_tree(flat)
    leaves = []
    for i in range(len(tree)):
        if len(tree[i]) == 1:
            leaves.append(i)
    return leaves


async def run_experiment(
    deployment: hydro.Deployment,
    profile,
    machine_pool,
    experiment_id,
    summaries_file,
    tree_arg,
    depth_arg,
    clients_arg,
    is_gcp,
    gcp_vpc,
):
    tree_depth = int(depth_arg)
    is_tree = tree_arg == "topolo"  # or "pn"

    num_replicas = 2**tree_depth - 1

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
                    network=gcp_vpc,
                )
            else:
                out = localhost_machine
            currently_deployed.append(out)
            return out

    all_nodes = []
    neighbors = get_neighbors_in_binary_tree(list(range(num_replicas)))
    cluster = [
        deployment.HydroflowCrate(
            src=str(
                (Path(__file__).parent.parent / "hydro_cli_examples").absolute()
            ),
            profile=profile,
            example="pn_counter" if tree_arg == "pn" else "pn_counter_delta",
            args=[json.dumps(neighbors[i])] if is_tree else [json.dumps([i]), json.dumps([num_replicas])],
            on=create_machine(),
        )
        for i in range(num_replicas)
    ]

    for i in range(num_replicas):
        cluster[i].ports.to_peer.send_to(
            hydro.demux(
                {
                    j: cluster[j].ports.from_peer.merge()
                    for j in range(num_replicas)
                }
            )
        )

    all_nodes = cluster

    if is_tree:
        leaves = get_leaves_in_binary_tree(list(range(num_replicas)))
        source = cluster[leaves[0]].node
        dest = cluster[leaves[-1]].node
    else:
        source = cluster[0]
        dest = cluster[-1]

    for node in all_nodes:
        if node is not dest:
            node.ports.query_responses.send_to(hydro.null())
        if node is not source:
            hydro.null().send_to(node.ports.increment_requests)

    latency_measurer = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent / "hydro_cli_examples").absolute()),
        profile=profile,
        example="topolotree_latency_measure",
        args=[json.dumps([num_clients])],
        on=create_machine(),
    )

    latency_measurer.ports.increment_start_node.send_to(
        source.ports.increment_requests.merge()
    )
    dest.ports.query_responses.send_to(latency_measurer.ports.end_node_query)

    await deployment.deploy()

    print("deployed!")

    latency = []
    memory_per_node = [[] for _ in range(num_replicas)]
    throughput_raw = []

    throughput = []

    latency_stdout = await latency_measurer.stdout()

    memories_streams_with_index = [
        stream.map(await node.stdout(), lambda x, i=i: (i, x))
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
                    number = int(line_split[1])  # microseconds
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

    init_memory = [memory[0] for memory in memory_per_node]
    print("init memory:")
    summarize(init_memory, "init_memory")

    final_memory = [memory[-1] for memory in memory_per_node]
    print("final memory:")
    summarize(final_memory, "final_memory")

    pd.DataFrame(latency).to_csv(
        "latency_"
        + tree_arg
        + "_tree_depth_"
        + str(tree_depth)
        + "_num_clients_"
        + str(num_clients)
        + "_"
        + experiment_id
        + ".csv",
        index=False,
        header=["latency"],
    )
    pd.DataFrame(throughput_raw).to_csv(
        "throughput_"
        + tree_arg
        + "_tree_depth_"
        + str(tree_depth)
        + "_num_clients_"
        + str(num_clients)
        + "_"
        + experiment_id
        + ".csv",
        index=False,
        header=["count", "period"],
    )
    pd.DataFrame(init_memory).to_csv(
        "init_memory_"
        + tree_arg
        + "_tree_depth_"
        + str(tree_depth)
        + "_num_clients_"
        + str(num_clients)
        + "_"
        + experiment_id
        + ".csv",
        index=False,
        header=["memory"],
    )
    pd.DataFrame(final_memory).to_csv(
        "final_memory_"
        + tree_arg
        + "_tree_depth_"
        + str(tree_depth)
        + "_num_clients_"
        + str(num_clients)
        + "_"
        + experiment_id
        + ".csv",
        index=False,
        header=["memory"],
    )

    for machine in currently_deployed:
        machine_pool.append(machine)


# hydro deploy toplotree_latency.hydro.py -- local/gcp once/pn/pn_counter_delta DEPTH_OF_TREE NUM_CLIENTS
async def main(args):
    # the current timestamp
    import datetime

    experiment_id = str(datetime.datetime.now())

    summaries_file = open(f"summaries_{experiment_id}.csv", "w")
    summaries_file.write(
        "protocol,tree_depth,num_clients,kind,mean,std,min,max,percentile_99,percentile_75,percentile_50,percentile_25,percentile_1"
    )

    deployment = hydro.Deployment()
    pool = []

    network = (
        hydro.GCPNetwork(
            project="hydro-chrisdouglas",
        )
        if args[0] == "gcp"
        else None
    )

    for depth_arg in args[2].split(","):
        for tree_arg in args[1].split(","):
            for num_clients_arg in args[3].split(","):
                await run_experiment(
                    deployment,
                    "dev" if args[0] == "local" else None,
                    pool,
                    experiment_id,
                    summaries_file,
                    tree_arg,
                    depth_arg,
                    num_clients_arg,
                    args[0] == "gcp",
                    network,
                )

    summaries_file.close()


if __name__ == "__main__":
    import sys
    import hydro.async_wrapper

    hydro.async_wrapper.run(main, sys.argv[1:])
