from codecs import decode
import json
from pathlib import Path
import pytest
import hydro

@pytest.mark.asyncio
async def test_networked_basic():
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    sender = deployment.CustomService(
        external_ports=[],
        on=localhost_machine.client_only(),
    )

    program_zero = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent).absolute()),
        args=["0"],
        example="networked_basic",
        profile="dev",
        on=localhost_machine
    )

    program_one = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent).absolute()),
        args=["1"],
        example="networked_basic",
        profile="dev",
        on=localhost_machine
    )

    sender_port = sender.client_port()
    sender_port.send_to(program_zero.ports.node_zero_input)

    program_zero.ports.zero_to_one.send_to(program_one.ports.zero_to_one)

    await deployment.deploy()

    receiver_out = await program_one.stdout()
    connection = await (await sender_port.server_port()).into_sink()

    await deployment.start()
    await connection.send(bytes("hi!", "utf-8"))

    async for log in receiver_out:
        assert log == "node one received: \"hi!\""
        break
