import hydro

async def main():
    deployment = hydro.Deployment()
    machine = deployment.Localhost()

    await machine.provision()

    program = deployment.HydroflowCrate(
        src=".",
        on=machine
    )

    program2 = deployment.HydroflowCrate(
        src=".",
        on=machine
    )

    program.ports.foo.send_to(program2.ports.bar)

    await deployment.deploy()
