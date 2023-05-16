import asyncio
import hydro
from pathlib import Path

async def main(args):
    machine_gcp = args[0] == "gcp"

    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    gcp_vpc = hydro.GCPNetwork(
        project="autocompartmentalization",
    )

    machine = deployment.GCPComputeEngineHost(
        project="autocompartmentalization",
        machine_type="e2-micro",
        image="debian-cloud/debian-11",
        region="us-west1-a",
        network=gcp_vpc
    ) if machine_gcp else localhost_machine

    machine2 = deployment.GCPComputeEngineHost(
        project="autocompartmentalization",
        machine_type="e2-micro",
        image="debian-cloud/debian-11",
        region="us-west1-a",
        network=gcp_vpc
    ) if machine_gcp else localhost_machine

    echo_server = deployment.HydroflowCrate(
        src=str(Path(__file__).parent.absolute()),
        example="ws_chat_server",
        on=machine,
        external_ports=[8080]
    )

    logger_server = deployment.HydroflowCrate(
        src=str(Path(__file__).parent.absolute()),
        example="stdout_receiver",
        on=machine2,
        display_id="logger"
    )

    echo_server.ports.to_logger.send_to(logger_server.ports.echo)

    await deployment.deploy()

    await deployment.start()

    if machine_gcp:
        print(machine.external_ip)
    
    while True:
        await asyncio.sleep(1)

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
