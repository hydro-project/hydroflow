import hydro
import json

def gcp_machine(deployment):
    return deployment.GCPComputeEngineHost(
            project="autocompartmentalization",
            machine_type="e2-micro",
            image="debian-cloud/debian-11",
            region="us-west1-a"
        )

async def main(args):
    machine_1_gcp = args[0] == "gcp"
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    machine1 = gcp_machine(deployment=deployment) if machine_1_gcp else localhost_machine
    program = deployment.HydroflowCrate(
            src=".",
            example="dedalus_tick",
            on=machine1
        )

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
