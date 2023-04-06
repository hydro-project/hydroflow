from codecs import decode
import json
from pathlib import Path
import pytest
import hydro

@pytest.mark.asyncio
async def test_connect_to_self():
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    program = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="empty_program",
        on=localhost_machine
    )

    program.ports.out.send_to(program.ports.input)

    await deployment.deploy()
    await deployment.start()

@pytest.mark.asyncio
async def test_python_sender():
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    sender = deployment.CustomService(
        external_ports=[],
        on=localhost_machine.client_only(),
    )

    receiver = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="stdout_receiver",
        on=localhost_machine
    )

    sender_port_1 = sender.client_port()
    sender_port_1.send_to(receiver.ports.echo.merge())

    sender_port_2 = sender.client_port()
    sender_port_2.send_to(receiver.ports.echo.merge())

    await deployment.deploy()

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    receiver_out = await receiver.stdout()

    await deployment.start()

    sender_1_connection = await (await sender_port_1.server_port()).into_sink()
    sender_2_connection = await (await sender_port_2.server_port()).into_sink()

    await sender_1_connection.send(bytes("hi 1!", "utf-8"))

    async for log in receiver_out:
        assert log == "echo \"hi 1!\""
        break

    await sender_2_connection.send(bytes("hi 2!", "utf-8"))
    async for log in receiver_out:
        assert log == "echo \"hi 2!\""
        break

@pytest.mark.asyncio
async def test_python_receiver():
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    sender = deployment.HydroflowCrate(
        src=str((Path(__file__).parent.parent.parent / "hydro_cli_examples").absolute()),
        example="dedalus_sender",
        args=[json.dumps(([0], 123))],
        on=localhost_machine
    )

    receiver = deployment.CustomService(
        external_ports=[],
        on=localhost_machine.client_only(),
    )

    receiver_port_0 = receiver.client_port()
    sender.ports.broadcast.send_to(hydro.demux({
        0: receiver_port_0,
    }))

    await deployment.deploy()
    await deployment.start()

    receiver_0_connection = await (await receiver_port_0.server_port()).into_source()
    async for received in receiver_0_connection:
        assert decode(received[8:], "utf-8") == "Hello 123"
        break
