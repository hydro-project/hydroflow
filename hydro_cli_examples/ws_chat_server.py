import asyncio
from typing import Dict, List
import hydro
from pathlib import Path

async def main(args):
    machine_gcp = args[0] == "gcp"
    num_replicas = int(args[1]) if len(args) > 1 else 1

    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    gcp_vpc = hydro.GCPNetwork(
        project="autocompartmentalization",
    )

    machines: List[hydro.GCPComputeEngineHost] = []
    chat_servers: Dict[int, hydro.HydroflowCrate] = {}
    for i in range(num_replicas):
        machine = deployment.GCPComputeEngineHost(
            project="autocompartmentalization",
            machine_type="e2-micro",
            image="debian-cloud/debian-11",
            region="us-west1-a",
            network=gcp_vpc
        ) if machine_gcp else localhost_machine

        machines.append(machine)

        chat_servers[i] = deployment.HydroflowCrate(
            src=str(Path(__file__).parent.absolute()),
            example="ws_chat_server",
            on=machine,
            external_ports=[8080 if machine_gcp else (8080 + i)],
            args=[str(num_replicas), str(i)]
        )

    for server in chat_servers.values():
        server.ports.to_peer.send_to(hydro.demux({
            i: server.ports.from_peer.merge()
            for (i, server) in chat_servers.items()
        }))

    await deployment.deploy()

    await deployment.start()

    if machine_gcp:
        for machine in machines:
            print(machine.external_ip)
    
    while True:
        await asyncio.sleep(1)

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
