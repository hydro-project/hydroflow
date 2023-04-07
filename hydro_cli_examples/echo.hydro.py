import hydro
import json

async def main(args):
    machine_1_gcp = args[0] == "gcp"
    machine_2_gcp = args[1] == "gcp"

    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    machine1 = deployment.GCPComputeEngineHost(
        project="autocompartmentalization",
        machine_type="e2-micro",
        image="debian-cloud/debian-11",
        region="us-west1-a"
    ) if machine_1_gcp else localhost_machine

    machine2 = deployment.GCPComputeEngineHost(
        project="autocompartmentalization",
        machine_type="e2-micro",
        image="debian-cloud/debian-11",
        region="us-west1-a"
    ) if machine_2_gcp else localhost_machine

    program = deployment.HydroflowCrate(
        src=".",
        example="dedalus_echo_leader",
        on=machine1
    )

    program2 = deployment.HydroflowCrate(
            src=".",
            example="dedalus_echo_participant",
            on=machine2
        )
    program2.ports.from_replica.send_to(hydro.demux({
        0: program.ports.from_replica.merge()
    }))
    program.ports.to_replica.send_to(hydro.demux({
        0: program2.ports.to_replica
    }))

    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    program_out = await program.stdout()

    await deployment.start()
    print("started!")

    counter = 0
    async for log in program_out:
        print(log)
        counter += 1
        if counter == 1000:
            break

    print(await program.exit_code())

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
