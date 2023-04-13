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

# rustup run nightly-2023-03-01-x86_64-unknown-linux-gnu hydro deploy ../hydro_cli_examples/toplotree_latency.hydro.py -- local/gcp DEPTH_OF_TREE
async def main(args):
    num_replicas = int(args[1])
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

    for node in cluster:
        hydro.null().send_to(node.ports.increment_requests.merge())

    source = cluster[0]
    dest = cluster[-1]

    def send_non_dest_queries_to_null(node):
        if node is not dest:
            node.ports.query_responses.send_to(hydro.null())

    for node in cluster:
        send_non_dest_queries_to_null(node)

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

    latency_stdout = await latency_measurer.stdout()
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

        for node in cluster:
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
        csv_file = open("pn_stats_"+ args[0] + "_replica_count_" + str(num_replicas) + "_" + experiment_id+".csv", "w")
        csv_file.write("mean,std,min,max,percentile_99,percentile_75,percentile_50,percentile_25,percentile_1\n")
        csv_file.write(str(np.mean(latency)) + "," + str(np.std(latency)) + "," + str(np.min(latency)) + "," + str(np.max(latency)) + "," + str(np.percentile(latency, 99)) + "," + str(np.percentile(latency, 75)) + "," + str(np.percentile(latency, 50)) + "," + str(np.percentile(latency, 25)) + "," + str(np.percentile(latency, 1)))
        csv_file.close()

        df = pd.DataFrame(latency)
        df.to_csv("pn_latency_"+ args[0] + "_replica_count_" + str(num_replicas) + "_" + experiment_id+".csv", index=False, header=False)
    finally:
        pass

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
