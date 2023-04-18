from codecs import decode
import json
from pathlib import Path
import pytest
import hydro

@pytest.mark.asyncio
async def test_server_sink():
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    sender = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="dedalus_sender",
        args=[json.dumps(([0, 1], 123))],
        on=localhost_machine.client_only()
    )

    receiver0 = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="dedalus_receiver",
        on=localhost_machine
    )

    receiver1 = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="dedalus_receiver",
        on=localhost_machine
    )

    sender.ports.broadcast.send_to(hydro.demux({
        0: receiver0.ports.broadcast.merge(),
        1: receiver1.ports.broadcast.merge(),
    }))

    await deployment.deploy()

    receiver0_out = await receiver0.stdout()
    receiver1_out = await receiver1.stdout()

    await deployment.start()

    async for log in receiver0_out:
        assert log == "echo (\"Hello 123\",)"
        break

    async for log in receiver1_out:
        assert log == "echo (\"Hello 123\",)"
        break

@pytest.mark.asyncio
async def test_client_sink():
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    sender = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="dedalus_sender",
        args=[json.dumps(([0, 1], 123))],
        on=localhost_machine
    )

    receiver0 = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="dedalus_receiver",
        on=localhost_machine.client_only()
    )

    receiver1 = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="dedalus_receiver",
        on=localhost_machine.client_only()
    )

    sender.ports.broadcast.send_to(hydro.demux({
        0: receiver0.ports.broadcast.merge(),
        1: receiver1.ports.broadcast.merge(),
    }))

    await deployment.deploy()

    receiver0_out = await receiver0.stdout()
    receiver1_out = await receiver1.stdout()

    await deployment.start()

    async for log in receiver0_out:
        assert log == "echo (\"Hello 123\",)"
        break

    async for log in receiver1_out:
        assert log == "echo (\"Hello 123\",)"
        break
