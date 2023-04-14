import asyncio
from codecs import decode
from typing import Optional
import hydro
import json
from pathlib import Path
from aiostream import stream

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

# rustup run nightly-2023-04-13-x86_64-unknown-linux-gnu hydro deploy ../hydro_cli_examples/toplotree_latency.hydro.py -- local/gcp DEPTH_OF_TREE
async def main(args):
    # the current timestamp
    import datetime
    experiment_id = str(datetime.datetime.now())

    summaries_file = open(f"summaries_{experiment_id}.csv", "w")
    summaries_file.write("is_tree,tree_depth,kind,mean,std,min,max,percentile_99,percentile_75,percentile_50,percentile_25,percentile_1")

    for tree_arg in args[1].split(","):
        is_tree = tree_arg == "topolo" # or "pn"

        for depth_arg in args[2].split(","):
            tree_depth = int(depth_arg)
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

            latency = []
            memory_per_node = [[] for _ in range(num_replicas)]
            throughput_raw = []

            throughput = []
            throughput_fig,throughput_ax = plt.subplots()
            throughput_plot = throughput_ax.plot(range(0, len(throughput)), throughput, label="throughput (ops/s)")[0]
            plt.legend()
            plt.xlabel("iterations")
            plt.ylabel("throughput (ops/s)")

            plt.show(block=False)

            latency_stdout = await latency_measurer.stdout()

            memories_streams_with_index = [
                stream.map(
                    await (await port.server_port()).into_source(),
                    lambda x,i=i: (i, x)
                )
                for i, port in enumerate(memories)
            ]

            async def memory_plotter():
                async with stream.merge(*memories_streams_with_index).stream() as merged:
                    async for node_idx, memory in merged:
                        memory = json.loads(decode(memory, "utf-8"))
                        memory_per_node[node_idx].append(memory)

            memory_plotter_task = asyncio.create_task(memory_plotter())

            async def latency_plotter():
                async for line in latency_stdout:
                    line_split = line.split(",")
                    if line_split[0] == "throughput":
                        count = int(line_split[1])
                        period = float(line_split[2])
                        throughput_raw.append([count, period])
                        throughput.append(count / period)

                        throughput_plot.set_xdata(range(0, len(throughput)))
                        throughput_plot.set_ydata(throughput)
                        throughput_ax.relim()
                        throughput_ax.autoscale_view()
                        throughput_fig.canvas.draw()
                        throughput_fig.canvas.flush_events()
                    elif line_split[0] == "latency":
                        number = int(line_split[1]) # microseconds
                        latency.append(number)

            latency_plotter_task = asyncio.create_task(latency_plotter())

            await deployment.start()
            print("started!")

            await asyncio.sleep(10)
            memory_plotter_task.cancel()
            latency_plotter_task.cancel()
            try:
                await memory_plotter_task
            except:
                pass
            try:
                await latency_plotter_task
            except:
                pass

            for node in all_nodes:
                await node.stop()

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
                summaries_file.write(str(is_tree) + ",")
                summaries_file.write(str(tree_depth) + ",")
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

            final_memories = [
                memory[-1]
                for memory in memory_per_node
            ]
            print("final memory:")
            summarize(final_memories, "final_memory")

            pd.DataFrame(latency).to_csv("latency_" + ("topolo" if is_tree else "pn") + "_tree_depth_" + str(tree_depth) + "_" + experiment_id+".csv", index=False, header=["latency"])
            pd.DataFrame(throughput_raw).to_csv("throughput_" + ("topolo" if is_tree else "pn") + "_tree_depth_" + str(tree_depth) + "_" + experiment_id+".csv", index=False, header=["count", "period"])
            pd.DataFrame(final_memories).to_csv("memory_" + ("topolo" if is_tree else "pn") + "_tree_depth_" + str(tree_depth) + "_" + experiment_id+".csv", index=False, header=["memory"])
    summaries_file.close()

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
