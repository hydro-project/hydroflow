import asyncio
from codecs import decode
from typing import Optional
import hydro
import json
from pathlib import Path
from aiostream import stream

import matplotlib
matplotlib.use('TkAgg')

import matplotlib.pyplot as plt
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

# rustup run nightly-2023-03-01-x86_64-unknown-linux-gnu hydro deploy ../hydro_cli_examples/toplotree_latency.hydro.py -- local/gcp DEPTH_OF_TREE
async def main(args):
    is_tree = args[1] == "topolo" # or "pn"
    tree_depth = int(args[2])
    num_replicas = 2 ** tree_depth - 1
    deployment = hydro.Deployment()

    localhost_machine = deployment.Localhost()

    gcp_vpc = hydro.GCPNetwork(
        project="hydro-chrisdouglas",
    )

    def create_machine():
        if args[0] == "gcp":
            return deployment.GCPComputeEngineHost(
                project="hydro-chrisdouglas",
                machine_type="e2-micro",
                image="debian-cloud/debian-11",
                region="us-west1-a",
                network=gcp_vpc
            )
        else:
            return localhost_machine

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
                example="pn_counter",
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

    memory_receiver = deployment.CustomService(
        on=localhost_machine,
        external_ports=[]
    )

    memories = []
    for node in all_nodes:
        hydro.null().send_to(node.ports.increment_requests.merge())
        receiver_port = memory_receiver.client_port()
        memories.append(receiver_port)
        node.ports.memory_report.send_to(receiver_port)

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

    latency_machine = deployment.GCPComputeEngineHost(
        project="hydro-chrisdouglas",
        machine_type="e2-micro",
        image="debian-cloud/debian-11",
        region="us-west1-a",
        network=gcp_vpc
    ) if args[0] == "gcp" else localhost_machine
    latency_measurer = deployment.HydroflowCrate(
        src=str(Path(__file__).parent.absolute()),
        example="topolotree_latency_measure",
        on=latency_machine
    )

    latency_measurer.ports.increment_start_node.send_to(source.ports.increment_requests.merge())
    dest.ports.query_responses.send_to(latency_measurer.ports.end_node_query)

    await deployment.deploy()

    print("deployed!")

    plt.ion()

    memories_streams_with_index = [
        stream.map(
            await (await port.server_port()).into_source(),
            lambda x,i=i: (i, x)
        )
        for i, port in enumerate(memories)
    ]

    async def memory_plotter():
        fig,ax = plt.subplots()

        memory_per_node = [[] for _ in range(num_replicas)]
        memory_plots = [
            ax.plot(range(0, len(memory_per_node[i])), memory_per_node[i], label=f"node {i}")[0]
            for i in range(num_replicas)
        ]
        plt.legend()
        plt.xlabel("iterations")
        plt.ylabel("memory (bytes)")
        fig.show()

        iter = 0
        try:
            async with stream.merge(*memories_streams_with_index).stream() as merged:
                async for node_idx, memory in merged:
                    iter += 1
                    memory = json.loads(decode(memory, "utf-8"))
                    memory_per_node[node_idx].append(memory)

                    for i in range(num_replicas):
                        memory_plot = memory_plots[i]
                        memory_plot.set_xdata(range(0, len(memory_per_node[i])))
                        memory_plot.set_ydata(memory_per_node[i])

                    ax.relim()
                    ax.autoscale_view()
        except asyncio.CancelledError:
            pass

    memory_plotter_task = asyncio.create_task(memory_plotter())
    latency_stdout = await latency_measurer.stdout()

    await deployment.start()
    print("started!")

    fig,ax = plt.subplots()

    latency = []
    latency_plot = ax.plot(range(0, len(latency)), latency, label="latency (mus)")[0]
    plt.legend()
    plt.xlabel("iterations")
    plt.ylabel("latency (mus)")
    fig.show()

    iter = 0
    async for line in latency_stdout:
        iter += 1
        number = int(line.split(",")[1]) # microseconds
        latency.append(number)

        if iter % 1000 == 0:
            latency_seconds = range(0, len(latency))
            latency_plot.set_xdata(latency_seconds)
            latency_plot.set_ydata(latency)

            ax.relim()
            ax.autoscale_view()
            plt.draw()
            plt.pause(0.01)
        if iter > 100000:
            break

    memory_plotter_task.cancel()
    await memory_plotter_task

    for node in all_nodes:
        await node.stop()

    # the current timestamp
    import datetime
    experiment_id = str(datetime.datetime.now())

    print("mean = ", np.mean(latency))
    print("std = ", np.std(latency))
    print("min = ", np.min(latency))
    print("max = ", np.max(latency))
    print("percentile 99 = ", np.percentile(latency, 99))
    print("percentile 75 = ", np.percentile(latency, 75))
    print("percentile 50 = ", np.percentile(latency, 50))
    print("percentile 25 = ", np.percentile(latency, 25))
    print("percentile 1 = ", np.percentile(latency, 1))

    # print the above values to a csv file
    csv_file = open("stats_"+ args[0] + "_tree_depth_" + str(tree_depth) + "_" + experiment_id+".csv", "w")
    csv_file.write("mean,std,min,max,percentile_99,percentile_75,percentile_50,percentile_25,percentile_1\n")
    csv_file.write(str(np.mean(latency)) + "," + str(np.std(latency)) + "," + str(np.min(latency)) + "," + str(np.max(latency)) + "," + str(np.percentile(latency, 99)) + "," + str(np.percentile(latency, 75)) + "," + str(np.percentile(latency, 50)) + "," + str(np.percentile(latency, 25)) + "," + str(np.percentile(latency, 1)))
    csv_file.close()

    df = pd.DataFrame(latency)
    df.to_csv("latency_"+ args[0] + "_tree_depth_" + str(tree_depth) + "_" + experiment_id+".csv", index=False, header=False)

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
