import hydro
import matplotlib.pyplot as plt
import matplotlib.text as text
import numpy as np
import json
import sys

def gcp_machine(deployment, gcp_vpc):
    return deployment.GCPComputeEngineHost(
        project="autocompartmentalization",
        machine_type="n2-standard-4",
        image="debian-cloud/debian-11",
        region="us-west1-a",
        network=gcp_vpc
    )

async def main(args):
    machine_1_gcp = args[0] == "gcp"
    machine_2_gcp = args[1] == "gcp"
    num_participants = int(args[2])
    flush_every_n = int(args[3])

    gcp_vpc = hydro.GCPNetwork(
        project="autocompartmentalization",
    )
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    machine1 = gcp_machine(deployment, gcp_vpc) if machine_1_gcp else localhost_machine
    machine2 = gcp_machine(deployment, gcp_vpc) if machine_2_gcp else localhost_machine

    program = deployment.HydroflowCrate(
        src=".",
        example="dedalus_vote_leader",
        args=[json.dumps([flush_every_n])],
        on=machine1
    )

    participant_programs = []
    participant_ports = {}
    for i in range(0, num_participants):
        program2 = deployment.HydroflowCrate(
            src=".",
            example="dedalus_vote_participant",
            args=[json.dumps([i])],
            on=machine2
        )
        program2.ports.from_replica.send_to(hydro.demux({
            0: program.ports.from_replica.merge()
        }))
        participant_programs.append(program2)
        participant_ports[i] = program2.ports.to_replica

    program.ports.to_replica.send_to(hydro.demux(participant_ports))

    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    program_out = await program.stdout()

    await deployment.start()
    print("started!")

    throughput = []

    plt.ion()               # interactive mode on
    fig,ax = plt.subplots()

    throughput_plot = ax.plot(range(0, len(throughput)), throughput, label="committed")[0]
    plt.legend()
    plt.xlabel("seconds")
    plt.ylabel("throughput")
    fig.show()

    try:
        async for log in program_out:
            split = log.split(",")
            if split[0] == "throughput":
                throughput.append(int(split[1]))
            print(log, file=sys.stderr)
            
            throughput_seconds = range(0, len(throughput))
            throughput_plot.set_xdata(throughput_seconds)
            throughput_plot.set_ydata(throughput)

            # Calculate slope
            if len(throughput_seconds) > 1:
                throughput_z = np.polyfit(throughput_seconds, throughput, 1)
                for x in plt.findobj(match=text.Text):
                    try:
                        x.remove()
                    except NotImplementedError:
                        pass
                plt.text(0.7, 0.1, "throughput=%.2f/s"%throughput_z[0], fontsize = 11, transform=ax.transAxes, horizontalalignment='right', verticalalignment='bottom')

            ax.relim()
            ax.autoscale_view()
            plt.draw()
            plt.pause(0.01)
    except:
        import traceback
        traceback.print_exc(file=sys.stderr)
        pass

    print(await program.exit_code())

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
