import hydro

async def main():
    deployment = hydro.Deployment()
    machine = deployment.Localhost()

    await machine.provision()

    program = deployment.HydroflowCrate(
        src=".",
        example="simple",
        on=machine
    )

    program2 = deployment.HydroflowCrate(
        src=".",
        example="simple",
        on=machine
    )

    program.ports.foo.send_to(program2.ports.bar)

    await deployment.deploy()
    
    async for log in program.stdout():
        print(log)

    async for log in program2.stdout():
        print(log)
