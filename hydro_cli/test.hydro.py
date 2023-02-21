import hydro

async def main():
    deployment = hydro.Deployment()
    machine = deployment.Localhost()

    program = deployment.HydroflowCrate(
        src=".",
        example="sender",
        on=machine
    )

    program2 = deployment.HydroflowCrate(
        src=".",
        example="receiver",
        on=machine
    )

    program.ports.foo.send_to(program2.ports.bar)

    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout
    program2out = await program2.stdout()
    counter = 0
    async for log in program2out:
        print(log)
        counter += 1
        if counter == 10:
            break

    print(await program.exit_code())
