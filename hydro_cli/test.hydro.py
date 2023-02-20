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

    counter = 0
    async for log in program2.stdout():
        print(log)
        counter += 1
        if counter == 10:
            break

    print(program.exit_code())
