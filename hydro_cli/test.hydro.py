import hydro

async def main():
    deployment = hydro.Deployment()
    machine = deployment.Localhost()

    program = deployment.HydroflowCrate(
        src="../hydroflow",
        example="cli_sender",
        features=["cli_integration"],
        on=machine
    )

    program2 = deployment.HydroflowCrate(
        src="../hydroflow",
        example="cli_receiver",
        features=["cli_integration"],
        on=machine
    )

    program.ports.foo.send_to(program2.ports.bar)

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
