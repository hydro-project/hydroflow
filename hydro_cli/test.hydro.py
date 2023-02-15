import hydro
from hydro import Localhost, HydroflowCrate

async def main():
    machine = Localhost()

    await machine.provision()

    program = HydroflowCrate(
        src=".",
        on=machine
    )

    program2 = HydroflowCrate(
        src=".",
        on=machine
    )

    program.ports.foo.send_to(program2.ports.bar)

    await hydro.deploy(program)
