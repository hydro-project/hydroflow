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
    vote_requester_gcp = args[1] == "gcp"
    participant_voter_gcp = args[2] == "gcp"
    committer_gcp = args[3] == "gcp"
    participant_acker_gcp = args[4] == "gcp"
    ender_gcp = args[5] == "gcp"
    num_participants = int(args[6])
    num_vote_requesters = int(args[7])
    num_participant_voter_partitions = int(args[8])
    num_committers = int(args[9])
    num_participant_acker_partitions = int(args[10])
    num_enders = int(args[11])
    
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
        example="dedalus_auto_2pc_coordinator",
        args=[json.dumps([num_vote_requesters])], # num_vote_requester_partitions
        on=coordinator_machine
    )

    participant_voter_start_ids = []
    for i in range(0, num_participants):
        participant_voter_start_ids.append(i * num_participant_voter_partitions)

    vote_requester_programs = []
    vote_to_vote_requester_ports = {}
    for i in range(0, num_vote_requesters):
        machine = gcp_machine(deployment, gcp_vpc) if vote_requester_gcp else localhost_machine
        vote_requester = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_2pc_vote_requester",
            args=[json.dumps((num_participant_voter_partitions, participant_voter_start_ids))], # num_participant_voters, participant_voter_start_ids
            on=machine
        )
        vote_requester_programs.append(vote_requester)
        vote_to_vote_requester_ports[i] = vote_requester.ports.vote_to_vote_requester

    participant_voter_programs = []
    vote_to_participant_ports = {}
    for i in range(0, num_participants * num_participant_voter_partitions):
        machine = gcp_machine(deployment, gcp_vpc) if participant_voter_gcp else localhost_machine
        participant = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_2pc_participant_voter",
            args=[json.dumps((i, num_committers, log_directory, str(i) + participant_log))], # my_id, num_committers, log_directory, participant_log
            on=machine
        )
        participant_voter_programs.append(participant)
        vote_to_participant_ports[i] = participant.ports.vote_to_participant.merge()
    
    participant_acker_start_ids = []
    for i in range(0, num_participants):
        participant_acker_start_ids.append(i * num_participant_acker_partitions)
    
    committer_programs = []
    vote_from_participant_ports = {}
    for i in range(0, num_committers):
        machine = gcp_machine(deployment, gcp_vpc) if committer_gcp else localhost_machine
        committer = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_2pc_committer",
            args=[json.dumps((num_participants, num_participant_acker_partitions, participant_acker_start_ids, log_directory, str(i) + coordinator_log))], # num_participants, num_participant_ackers, participant_acker_start_ids, log_directory, coordinator_log
            on=machine
        )
        committer_programs.append(committer)
        vote_from_participant_ports[i] = committer.ports.vote_from_participant.merge()
    
    participant_acker_programs = []
    commit_to_participant_ports = {}
    for i in range(0, num_participants * num_participant_acker_partitions):
        machine = gcp_machine(deployment, gcp_vpc) if participant_acker_gcp else localhost_machine
        participant = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_2pc_participant_acker",
            args=[json.dumps((i, num_enders))], # my_id, num_enders
            on=machine
        )
        participant_acker_programs.append(participant)
        commit_to_participant_ports[i] = participant.ports.commit_to_participant.merge()

    ender_programs = []
    ack_from_participant_ports = {}
    for i in range(0, num_enders):
        machine = gcp_machine(deployment, gcp_vpc) if ender_gcp else localhost_machine
        ender = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_2pc_ender",
            args=[json.dumps((i, num_participants))], # my_id, num_participants
            on=machine
        )
        ender_programs.append(ender)
        ack_from_participant_ports[i] = ender.ports.ack_from_participant.merge()

    # CONNECTIONS
    # coordinator -> vote_requester
    coordinator.ports.vote_to_vote_requester.send_to(hydro.demux(vote_to_vote_requester_ports))
    # vote_requester -> participant_voter
    for i in range(0, num_vote_requesters):
        vote_requester_programs[i].ports.vote_to_participant.send_to(hydro.demux(vote_to_participant_ports))
    # participant_voter -> committer
    for i in range(0, num_participants * num_participant_voter_partitions):
        participant_voter_programs[i].ports.vote_from_participant.send_to(hydro.demux(vote_from_participant_ports))
    # committer -> participant_acker
    for i in range(0, num_committers):
        committer_programs[i].ports.commit_to_participant.send_to(hydro.demux(commit_to_participant_ports))
    # participant_acker -> ender
    for i in range(0, num_participants * num_participant_acker_partitions):
        participant_acker_programs[i].ports.ack_from_participant.send_to(hydro.demux(ack_from_participant_ports))

    
    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    program_out = await ender_programs[0].stdout()

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
            if split[0] == "total_throughput" and split[1] == "0":
                total_throughput.append(int(split[2]) * num_enders)
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
