import hydro
import json
from pathlib import Path

async def main(args):
    machine_1_gcp = args[0] == "gcp"
    machine_2_gcp = args[1] == "gcp"

    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    gcp_vpc = hydro.GCPNetwork(
        project="autocompartmentalization",
    )

    machine1 = deployment.GCPComputeEngineHost(
        project="autocompartmentalization",
        machine_type="e2-micro",
        image="debian-cloud/debian-11",
        region="us-west1-a",
        network=gcp_vpc
    ) if machine_1_gcp else localhost_machine

    machine2 = deployment.GCPComputeEngineHost(
        project="autocompartmentalization",
        machine_type="e2-micro",
        image="debian-cloud/debian-11",
        region="us-west1-a",
        network=gcp_vpc
    ) if machine_2_gcp else localhost_machine

    sender = deployment.HydroflowCrate(
        src=str(Path(__file__).parent.absolute()),
        example="dedalus_sender",
        args=[json.dumps([0])],
        on=machine1
    )

    receiver = deployment.HydroflowCrate(
        src=str(Path(__file__).parent.absolute()),
        example="dedalus_receiver",
        on=machine2
    )

    sender.ports.broadcast.send_to(hydro.demux({
        0: receiver.ports.broadcast
    }))

    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    receiver_out = await receiver.stdout()

    await deployment.start()
    print("started!")

    counter = 0
    async for log in receiver_out:
        print(f"{counter}: {log}")
        counter += 1
        if counter == 10:
            break

    print(await sender.exit_code())

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
