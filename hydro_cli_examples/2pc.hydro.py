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
    coordinator_gcp = args[0] == "gcp"
    participants_gcp = args[1] == "gcp"
    num_participants = int(args[2])
    
    log_directory = "2pc_out"
    coordinator_log = "coordinator.txt"
    participant_log = "participant.txt"

    gcp_vpc = hydro.GCPNetwork(
        project="autocompartmentalization",
    )
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    coordinator_machine = gcp_machine(deployment, gcp_vpc) if coordinator_gcp else localhost_machine
    coordinator = deployment.HydroflowCrate(
        src=".",
        example="dedalus_2pc_coordinator",
        args=[json.dumps((num_participants, log_directory, coordinator_log))], # num_participants, log_directory, coordinator_log
        on=coordinator_machine
    )

    participant_programs = []
    participant_vote_ports = {}
    participant_commit_ports = {}
    for i in range(0, num_participants):
        machine = gcp_machine(deployment, gcp_vpc) if participants_gcp else localhost_machine
        participant = deployment.HydroflowCrate(
            src=".",
            example="dedalus_2pc_participant",
            args=[json.dumps((i, log_directory, str(i) + participant_log))], # my_id, log_directory, participant_log
            on=machine
        )
        participant.ports.vote_from_participant.send_to(hydro.demux({
            0: coordinator.ports.vote_from_participant.merge()
        }))
        participant.ports.ack_from_participant.send_to(hydro.demux({
            0: coordinator.ports.ack_from_participant.merge()
        }))
        participant_programs.append(participant)
        participant_vote_ports[i] = participant.ports.vote_to_participant
        participant_commit_ports[i] = participant.ports.commit_to_participant

    coordinator.ports.vote_to_participant.send_to(hydro.demux(participant_vote_ports))
    coordinator.ports.commit_to_participant.send_to(hydro.demux(participant_commit_ports))

    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    program_out = await coordinator.stdout()

    await deployment.start()
    print("started!")

    total_throughput = []

    plt.ion()               # interactive mode on
    fig,ax = plt.subplots()

    total_throughput_plot = ax.plot(range(0, len(total_throughput)), total_throughput, label="total throughput")[0]
    plt.legend()
    plt.xlabel("index in array")
    plt.ylabel("throughput")
    fig.show()

    try:
        async for log in program_out:
            split = log.split(",")
            if split[0] == "total_throughput":
                total_throughput.append(int(split[1]))
            print(log, file=sys.stderr)
            
            throughput_seconds = range(0, len(total_throughput))
            total_throughput_plot.set_xdata(throughput_seconds)
            total_throughput_plot.set_ydata(total_throughput)
            
            # Calculate slope
            if len(total_throughput) > 1:
                throughput_z = np.polyfit(throughput_seconds, total_throughput, 1)
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

    print(await coordinator.exit_code())

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
