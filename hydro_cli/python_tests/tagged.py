from codecs import decode
import json
from pathlib import Path
import pytest
import hydro

@pytest.mark.asyncio
async def test_server_sink():
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    sender0 = deployment.CustomService(
        external_ports=[],
        on=localhost_machine.client_only(),
    )

    sender1 = deployment.CustomService(
        external_ports=[],
        on=localhost_machine.client_only(),
    )

    receiver = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="tagged_stdout_receiver",
        on=localhost_machine
    )

    sender0_port = sender0.client_port()
    sender1_port = sender1.client_port()

    sender0_port.tagged(0).send_to(receiver.ports.echo.merge())
    sender1_port.tagged(1).send_to(receiver.ports.echo.merge())

    await deployment.deploy()

    receiver_out = await receiver.stdout()

    await deployment.start()

    sender0_connection = await (await sender0_port.server_port()).into_sink()
    sender1_connection = await (await sender1_port.server_port()).into_sink()

    await sender0_connection.send("hello from 0".encode())
    async for log in receiver_out:
        assert log == "echo (0, \"hello from 0\")"
        break

    await sender1_connection.send("hello from 1".encode())
    async for log in receiver_out:
        assert log == "echo (1, \"hello from 1\")"
        break

@pytest.mark.asyncio
async def test_client_sink_demux():
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    sender0 = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="dedalus_sender",
        args=[json.dumps(([0], 123))],
        on=localhost_machine
    )

    sender1 = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="dedalus_sender",
        args=[json.dumps(([0], 124))],
        on=localhost_machine
    )

    receiver = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="tagged_stdout_receiver",
        on=localhost_machine.client_only()
    )

    sender0.ports.broadcast.tagged(0).send_to(hydro.demux({
        0: receiver.ports.echo.merge()
    }))
    sender1.ports.broadcast.tagged(1).send_to(hydro.demux({
        0: receiver.ports.echo.merge()
    }))

    await deployment.deploy()

    receiver_out = await receiver.stdout()

    await deployment.start()

    got_zero = False
    got_one = False
    async for log in receiver_out:
        if log.startswith("echo (0"):
            assert log.endswith("123\")")
            got_zero = True
        elif log.startswith("echo (1"):
            assert log.endswith("124\")")
            got_one = True
        else:
            assert False

        if got_zero and got_one:
            break
