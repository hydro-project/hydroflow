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
    leader_gcp = args[0] == "gcp"
    broadcaster_gcp = args[1] == "gcp"
    collector_gcp = args[2] == "gcp"
    participant_gcp = args[3] == "gcp"
    num_broadcasters = int(args[4])
    num_collectors = int(args[5])
    num_participants = int(args[6])
    num_participant_partitions = int(args[7]) # Total number of participants = num_participants * num_participant_partitions

    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    leader_machine = gcp_machine(deployment=deployment) if leader_gcp else localhost_machine
    leader_program = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_vote_leader",
            args=[json.dumps([num_broadcasters])],
            on=leader_machine
        )

    broadcaster_machines = []
    broadcaster_programs = []
    to_broadcaster_ports = {}
    participant_start_ids = []
    for i in range(0, num_participants):
        participant_start_ids.append(i * num_participant_partitions)
        
    for i in range(0, num_broadcasters):
        broadcaster_machine = gcp_machine(deployment=deployment) if broadcaster_gcp else localhost_machine
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
        collector_machine = gcp_machine(deployment=deployment) if collector_gcp else localhost_machine
        program = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_vote_collector",
            args=[json.dumps([num_participants])],
            on=collector_machine
        )
        collector_machines.append(collector_machine)
        collector_programs.append(program)
        to_collector_ports[i] = program.ports.from_participant.merge()

    participant_machines = []
    participant_programs = []
    to_participant_ports = {}
    for i in range(0, num_participants * num_participant_partitions):
        participant_machine = gcp_machine(deployment=deployment) if participant_gcp else localhost_machine
        program = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_vote_participant",
            args=[json.dumps((i, num_collectors))],
            on=participant_machine
        )
        participant_machines.append(participant_machine)
        participant_programs.append(program)
        to_participant_ports[i] = program.ports.to_participant


    leader_program.ports.to_broadcaster.send_to(hydro.demux(to_broadcaster_ports))
    for i in range(0, num_broadcasters):
        broadcaster_programs[i].ports.to_participant.send_to(hydro.demux(to_participant_ports))
    for i in range(0, num_participants * num_participant_partitions):
        participant_programs[i].ports.from_participant.send_to(hydro.demux(to_collector_ports))
    

    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    program_out = await leader_program.stdout()

    await deployment.start()
    print("started!")

    counter = 0
    async for log in program_out:
        print(log)
        counter += 1
        if counter == 1000:
            break

    print(await leader_program.exit_code())

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
