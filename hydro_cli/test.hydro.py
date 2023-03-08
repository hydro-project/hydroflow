import hydro

async def main(args):
    gcp = args[0] == "gcp"

    deployment = hydro.Deployment()
    machine1 = deployment.GCPComputeEngineHost(
        project="autocompartmentalization",
        machine_type="e2-micro",
        image="debian-cloud/debian-11",
        region="us-west1-a"
    ) if gcp else deployment.Localhost()

    machine2 = deployment.GCPComputeEngineHost(
        project="autocompartmentalization",
        machine_type="e2-micro",
        image="debian-cloud/debian-11",
        region="us-west1-a"
    ) if gcp else machine1

    program = deployment.HydroflowCrate(
        src="../hydroflow",
        example="cli_sender",
        features=["cli_integration"],
        on=machine1
    )

    program2 = deployment.HydroflowCrate(
        src="../hydroflow",
        example="cli_receiver",
        features=["cli_integration"],
        on=machine2
    )

    program.ports.foo.send_to(hydro.demux({
        0: program2.ports.bar
    }))

    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    program2out = await program2.stdout()

    await deployment.start()
    print("started!")

    counter = 0
    async for log in program2out:
        print(f"{counter}: {log}")
        counter += 1
        if counter == 10:
            break

    print(await program.exit_code())
