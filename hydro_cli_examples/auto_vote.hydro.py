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
    leader_gcp = args[0] == "gcp"
    broadcaster_gcp = args[1] == "gcp"
    collector_gcp = args[2] == "gcp"
    participant_gcp = args[3] == "gcp"
    num_broadcasters = int(args[4])
    num_collectors = int(args[5])
    num_participants = int(args[6])
    num_participant_partitions = int(args[7]) # Total number of participants = num_participants * num_participant_partitions
    flush_every_n = int(args[7])

    gcp_vpc = hydro.GCPNetwork(
        project="autocompartmentalization",
    )
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    leader_machine = gcp_machine(deployment, gcp_vpc) if leader_gcp else localhost_machine
    leader_program = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_vote_leader",
            args=[json.dumps((num_broadcasters, flush_every_n))],
            on=leader_machine
        )

    broadcaster_machines = []
    broadcaster_programs = []
    to_broadcaster_ports = {}
    participant_start_ids = []
    for i in range(0, num_participants):
        participant_start_ids.append(i * num_participant_partitions)
        
    for i in range(0, num_broadcasters):
        broadcaster_machine = gcp_machine(deployment, gcp_vpc) if broadcaster_gcp else localhost_machine
        program = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_vote_broadcaster",
            args=[json.dumps((participant_start_ids,num_participant_partitions))],
            on=broadcaster_machine
        )
        broadcaster_machines.append(broadcaster_machine)
        broadcaster_programs.append(program)
        to_broadcaster_ports[i] = program.ports.to_broadcaster

    collector_machines = []
    collector_programs = []
    to_collector_ports = {}
    for i in range(0, num_collectors):
        collector_machine = gcp_machine(deployment, gcp_vpc) if collector_gcp else localhost_machine
        program = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_vote_collector",
            args=[json.dumps((i, num_participants))],
            on=collector_machine
        )
        collector_machines.append(collector_machine)
        collector_programs.append(program)
        to_collector_ports[i] = program.ports.from_participant.merge()

    participant_machines = []
    participant_programs = []
    to_participant_ports = {}
    for i in range(0, num_participants * num_participant_partitions):
        participant_machine = gcp_machine(deployment, gcp_vpc) if participant_gcp else localhost_machine
        program = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_vote_participant",
            args=[json.dumps((i, num_collectors))],
            on=participant_machine
        )
        participant_machines.append(participant_machine)
        participant_programs.append(program)
        to_participant_ports[i] = program.ports.to_participant.merge()


    leader_program.ports.to_broadcaster.send_to(hydro.demux(to_broadcaster_ports))
    for i in range(0, num_broadcasters):
        broadcaster_programs[i].ports.to_participant.send_to(hydro.demux(to_participant_ports))
    for i in range(0, num_participants * num_participant_partitions):
        participant_programs[i].ports.from_participant.send_to(hydro.demux(to_collector_ports))
    

    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    program_out = await collector_programs[0].stdout()

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
            if split[0] == "throughput" and split[1] == "0":
                throughput.append(int(split[2]) * num_collectors)
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

    print(await leader_program.exit_code())

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
