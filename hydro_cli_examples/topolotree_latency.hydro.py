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

# rustup run nightly-2023-03-01-x86_64-unknown-linux-gnu hydro deploy ../hydro_cli_examples/toplotree_latency.hydro.py -- local/gcp DEPTH_OF_TREE
async def main(args):
    tree_depth = int(args[1])
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

    tree = create_tree(tree_depth, deployment, create_machine)
    tree.node.ports.to_parent.send_to(hydro.null())
    hydro.null().send_to(tree.node.ports.from_parent)

    def create_increment_port(node):
        hydro.null().send_to(node.ports.increment_requests.merge())

    tree.map(create_increment_port)

    source = tree
    while source.left is not None:
        source = source.left
    dest = tree
    while dest.right is not None:
        dest = dest.right

    def send_non_dest_queries_to_null(node):
        if node is not dest.node:
            node.ports.query_responses.send_to(hydro.null())

    tree.map(send_non_dest_queries_to_null)

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

    latency_measurer.ports.increment_start_node.send_to(source.node.ports.increment_requests.merge())
    dest.node.ports.query_responses.send_to(latency_measurer.ports.end_node_query)

    await deployment.deploy()

    async def get_stdouts(node):
        return await node.stdout()
    tree_stdouts = await tree.map_async(get_stdouts)
    print("deployed!")

    with_stdouts = [
        stream.map(stdout, lambda x,path=path: (path, x))
        for (stdout, path) in tree_stdouts.flatten_with_path()
    ]

    async def print_stdouts():
        try:
            async with stream.merge(*with_stdouts).stream() as merged:
                async for _ in merged:
                    pass
        except asyncio.CancelledError:
            pass
    
    latency_stdout = await latency_measurer.stdout()

    print_stdout_task = asyncio.create_task(print_stdouts())
    try:
        await deployment.start()
        print("started!")

        plt.ion()               # interactive mode on
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
            if iter > 10000:
                break

        await tree.map_async(lambda node: node.stop())

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
    finally:
        print_stdout_task.cancel()
        try:
            await print_stdout_task
        except:
            pass

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
