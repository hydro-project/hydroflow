import hydro
import json
from pathlib import Path
from aiostream import stream

async def main(args):

    deployment = hydro.Deployment()

    machine1 = deployment.PodHost()

    machine2 = deployment.PodHost()

    sender_count = 2
    senders = [deployment.HydroflowCrate(
        src=str(Path(__file__).parent.absolute()),
        example="dedalus_sender",
        args=[json.dumps(([0, 1], i))],
        on=machine1
    ) for i in range(sender_count)]

    receiver1 = deployment.HydroflowCrate(
        src=str(Path(__file__).parent.absolute()),
        example="dedalus_receiver",
        on=machine2
    )

    receiver2 = deployment.HydroflowCrate(
        src=str(Path(__file__).parent.absolute()),
        example="dedalus_receiver",
        on=machine2
    )

    for sender in senders:
        sender.ports.broadcast.send_to(hydro.demux({
            0: receiver1.ports.broadcast.merge(),
            1: receiver2.ports.broadcast.merge()
        }))

    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    receiver_1_out = await receiver1.stdout()
    receiver_2_out = await receiver2.stdout()

    await deployment.start()
    print("started!")

    counter = 0
    async with stream.merge(stream.map(receiver_1_out, lambda x: f"RECEIVER 1: {x}"), stream.map(receiver_2_out, lambda x: f"RECEIVER 2: {x}")).stream() as merged:
        async for log in merged:
            print(log)
            counter += 1
            if counter == 10:
                break

    for sender in senders:
        await sender.stop()
        print(await sender.exit_code())

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
