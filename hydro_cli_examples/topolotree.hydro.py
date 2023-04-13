import asyncio
from codecs import decode
from typing import Optional
import hydro
import json
from pathlib import Path
from aiostream import stream

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

# rustup run nightly-2023-03-01-x86_64-unknown-linux-gnu hydro deploy ../hydro_cli_examples/toplotree.hydro.py -- local (or gcp)
async def main(args):
    tree_depth = int(args[1])
    deployment = hydro.Deployment()

    localhost_machine = deployment.Localhost()

    python_sender = deployment.CustomService(
        external_ports=[],
        on=localhost_machine,
    )

    gcp_vpc = hydro.GCPNetwork(
        # project="autocompartmentalization",
        project="hydro-chrisdouglas",
    )

    def create_machine():
        if args[0] == "gcp":
            return deployment.GCPComputeEngineHost(
                # project="autocompartmentalization",
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
        port = python_sender.client_port()
        port.send_to(node.ports.increment_requests)
        return port

    tree_increment_ports = tree.map(create_increment_port)

    def create_query_response_port(node):
        port = python_sender.client_port()
        node.ports.query_responses.send_to(port)
        return port

    tree_query_response_ports = tree.map(create_query_response_port)

    await deployment.deploy()

    async def get_increment_channel(port):
        return await (await port.server_port()).into_sink()
    tree_increment_channels = await tree_increment_ports.map_async(get_increment_channel)

    async def get_query_response_channel(port):
        return await (await port.server_port()).into_source()
    tree_query_response_channels = await tree_query_response_ports.map_async(get_query_response_channel)

    async def get_stdouts(node):
        return await node.stdout()
    tree_stdouts = await tree.map_async(get_stdouts)

    def stream_printer(path, v):
        parsed = json.loads(decode(v, "utf-8"))
        return f"{path}: {parsed}"

    print("deployed!")

    with_path_responses = [
        stream.map(response, lambda x,path=path: stream_printer(path, x))
        for (response, path) in tree_query_response_channels.flatten_with_path()
    ]

    # async def print_queries():
    #     try:
    #         async with stream.merge(*with_path_responses).stream() as merged:
    #             async for log in merged:
    #                 print(log)
    #     except asyncio.CancelledError:
    #         pass

    with_stdouts = [
        stream.map(stdout, lambda x,path=path: (path, x))
        for (stdout, path) in tree_stdouts.flatten_with_path()
    ]

    async def print_stdouts():
        try:
            async with stream.merge(*with_stdouts).stream() as merged:
                async for path, log in merged:
                    if not log.startswith("to_query"):
                        return f"{path}: {log}"
        except asyncio.CancelledError:
            pass
    
    # print_query_task = asyncio.create_task(print_queries())
    print_stdout_task = asyncio.create_task(print_stdouts())
    try:
        await deployment.start()
        print("started!")

        await asyncio.sleep(1)

        for i in range(1000000):
            if i % 10000 == 0:
                print(f"sending increment {i}")
            await tree_increment_channels.node.send(bytes("{\"tweet_id\": " + str(i) + ", \"likes\": " + str(i % 2 * 2 - 1) + "}", "utf-8"))
            # print("temp")
            # await tree_increment_channels.node.send(bytes(F"""{"tweet_id": 1, "likes": {i % 2}}""", "utf-8"))
            #await asyncio.sleep(1)
    finally:
        print_query_task.cancel()
        print_stdout_task.cancel()
        await print_query_task
        await print_stdout_task

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
