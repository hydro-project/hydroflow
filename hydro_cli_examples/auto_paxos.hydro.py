import hydro
import json

def gcp_machine(deployment, gcp_vpc):
    return deployment.GCPComputeEngineHost(
        project="autocompartmentalization",
        machine_type="e2-micro",
        image="debian-cloud/debian-11",
        region="us-west1-a",
        network=gcp_vpc
    )

async def main(args):
    proposer_gcp = args[0] == "gcp"
    acceptor_gcp = args[1] == "gcp"
    p2a_proxy_leader_gcp = args[2] == "gcp"
    p2b_proxy_leader_gcp = args[3] == "gcp"
    coordinator_gcp = args[4] == "gcp"
    f = int(args[5])
    p1a_node_0_timeout = int(args[6])
    p1a_other_nodes_timeout = int(args[7])
    i_am_leader_resend_timeout = int(args[8])
    i_am_leader_check_timeout = int(args[9])
    num_p2a_proxy_leaders = int(args[10])
    num_p2b_proxy_leaders = int(args[11])
    num_acceptor_groups = int(args[12])
    # i_am_leader_check_timeout should >> i_am_leader_resend_timeout, so the current leader has time to send a heartbeat
    # Leader election time (out of our control) should >> p1a_timeout, so the leader doesn't spam acceptors. p1a_timeout should differ between proposers to avoid contention

    gcp_vpc = hydro.GCPNetwork(
        project="autocompartmentalization",
    )
    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    # set up proposer
    proposer_machine = []
    proposer_programs = []
    proposer_p1b_ports = {}
    proposer_p1b_log_ports = {}
    proposer_p2b_ports = {}
    proposer_inputs_ports = {}
    proposer_i_am_leader_ports = {}
    for i in range(0, f+1):
        proposer_i_am_leader_ports[i] = {}
    for i in range(0, f+1):
        machine = gcp_machine(deployment=deployment, gcp_vpc=gcp_vpc) if proposer_gcp else localhost_machine
        proposer_machine.append(machine)
        p1a_timeout = p1a_node_0_timeout if i == 0 else p1a_other_nodes_timeout # proposer with id 0 is much more likely to be the leader

        proposer = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_paxos_proposer",
            args=[json.dumps((i, f, num_acceptor_groups, num_p2a_proxy_leaders, p1a_timeout, i_am_leader_resend_timeout, i_am_leader_check_timeout))], # my_id, f, num_acceptor_groups, num_p2a_proxy_leaders, p1a_timeout_const, i_am_leader_resend_timeout_const, i_am_leader_check_timeout_const
            on=machine
        )
        proposer_programs.append(proposer)
        proposer_p1b_ports[i] = proposer.ports.p1b.merge()
        proposer_p1b_log_ports[i] = proposer.ports.p1b_log.merge()
        proposer_p2b_ports[i] = proposer.ports.p2b.merge()
        proposer_inputs_ports[i] = proposer.ports.inputs.merge()
        for j in range(0, f+1):
            if i != j: # don't let proposers send to themselves
                proposer_i_am_leader_ports[j][i] = proposer.ports.i_am_leader_source.merge()

    acceptor_start_ids = []
    for i in range(0, 2*f+1):
        acceptor_start_ids.append(i * num_acceptor_groups)

    # set up p2a proxy leaders
    p2a_proxy_leader_machines = []
    p2a_proxy_leader_programs = []
    to_p2a_proxy_leader_ports = {}
    for proposer_id in range(0, f+1):
        p2a_proxy_leader_machines.append([])
        p2a_proxy_leader_programs.append([])
        for i in range(0, num_p2a_proxy_leaders):
            machine = gcp_machine(deployment=deployment, gcp_vpc=gcp_vpc) if p2a_proxy_leader_gcp else localhost_machine
            p2a_proxy_leader_machines[proposer_id].append(machine)
            p2a_proxy_leader_id = proposer_id*num_p2a_proxy_leaders + i
            
            p2a_proxy_leader = deployment.HydroflowCrate(
                src=".",
                example="dedalus_auto_paxos_p2a_proxy",
                args=[json.dumps((p2a_proxy_leader_id, acceptor_start_ids, num_acceptor_groups))], # my_id, acceptor_start_ids, num_acceptor_groups
                on=machine
            )
            p2a_proxy_leader_programs[proposer_id].append(p2a_proxy_leader)
            to_p2a_proxy_leader_ports[p2a_proxy_leader_id] = p2a_proxy_leader.ports.p2a_to_proxy.merge()

    # set up acceptors
    acceptor_machines = []
    acceptor_programs = []
    acceptor_p1a_ports = {}
    acceptor_p2a_ports = {}
    acceptor_p1a_commit_ports = {}
    for acceptor_id in range(0, 2*f+1):
        acceptor_machines.append([])
        acceptor_programs.append([])
        for i in range(0, num_acceptor_groups):
            machine = gcp_machine(deployment=deployment, gcp_vpc=gcp_vpc) if acceptor_gcp else localhost_machine
            acceptor_machines[acceptor_id].append(machine)
            partition_id = acceptor_id*num_acceptor_groups + i
            
            coordinator = acceptor_id # Each group of acceptors shares 1 coordinator
            acceptor = deployment.HydroflowCrate(
                src=".",
                example="dedalus_auto_paxos_acceptor",
                args=[json.dumps([acceptor_id, partition_id, coordinator, num_p2b_proxy_leaders])], # acceptor_id, partition_id, coordinator, num_p2b_proxy_leaders
                on=machine
            )
            acceptor_programs[acceptor_id].append(acceptor)
            acceptor_p1a_ports[partition_id] = acceptor.ports.p1a.merge()
            acceptor_p2a_ports[partition_id] = acceptor.ports.p2a.merge()
            acceptor_p1a_commit_ports[partition_id] = acceptor.ports.p1a_commit.merge()

    # set up p2b proxy leaders
    p2b_proxy_leader_machines = []
    p2b_proxy_leader_programs = []
    to_p2b_proxy_leader_ports = {}
    for proposer_id in range(0, f+1):
        p2b_proxy_leader_machines.append([])
        p2b_proxy_leader_programs.append([])
        for i in range(0, num_p2b_proxy_leaders):
            machine = gcp_machine(deployment=deployment, gcp_vpc=gcp_vpc) if p2b_proxy_leader_gcp else localhost_machine
            p2b_proxy_leader_machines[proposer_id].append(machine) 
            p2b_proxy_leader_id = proposer_id*num_p2b_proxy_leaders + i

            p2b_proxy_leader = deployment.HydroflowCrate(
                src=".",
                example="dedalus_auto_paxos_p2b_proxy",
                args=[json.dumps((p2b_proxy_leader_id, f, acceptor_start_ids, num_acceptor_groups, num_p2b_proxy_leaders, proposer_id))], # my_id, f, acceptor_start_ids, num_acceptor_groups, num_p2b_proxy_leaders, proposer
                on=machine
            )
            p2b_proxy_leader_programs[proposer_id].append(p2b_proxy_leader)
            to_p2b_proxy_leader_ports[p2b_proxy_leader_id] = p2b_proxy_leader.ports.p2b.merge()

    partitions_of_acceptor = []
    for acceptor_id in range(0, 2*f+1):
        partitions_of_acceptor.append([])
        for i in range(0, num_acceptor_groups):
            partitions_of_acceptor[acceptor_id].append(acceptor_id*num_acceptor_groups + i)

    # set up coordinators
    coordinator_machines = []
    coordinator_programs = []
    to_coordinator_p1a_ports = {}
    for i in range(0, 2*f+1):
        machine = gcp_machine(deployment=deployment, gcp_vpc=gcp_vpc) if coordinator_gcp else localhost_machine
        coordinator_machines.append(machine)
        
        coordinator = deployment.HydroflowCrate(
            src=".",
            example="dedalus_auto_paxos_coordinator",
            args=[json.dumps((i, num_acceptor_groups, partitions_of_acceptor[i]))], # my_id, num_acceptor_groups, acceptors
            on=machine
        )
        coordinator_programs.append(coordinator)
        to_coordinator_p1a_ports[i] = coordinator.ports.p1a_vote.merge()

    # CONNECTIONS
    for i in range(0, f+1):
        # proposer -> acceptor, p1a
        proposer_programs[i].ports.p1a.send_to(hydro.demux(acceptor_p1a_ports))
        # proposer -> proposer, i_am_leader
        proposer_programs[i].ports.i_am_leader_sink.send_to(hydro.demux(proposer_i_am_leader_ports[i]))
        # proposer -> p2a_proxy_leader, p2a
        proposer_programs[i].ports.p2a.send_to(hydro.demux(to_p2a_proxy_leader_ports))
    for acceptor_id in range(0, 2*f+1):
        for i in range(0, num_acceptor_groups):
            # acceptor -> coordinator, p1a_vote
            acceptor_programs[acceptor_id][i].ports.p1a_vote.send_to(hydro.demux(to_coordinator_p1a_ports))
            # acceptor -> proposer, p1b
            acceptor_programs[acceptor_id][i].ports.p1b.send_to(hydro.demux(proposer_p1b_ports))
            # acceptor -> proposer, p1b_log
            acceptor_programs[acceptor_id][i].ports.p1b_log.send_to(hydro.demux(proposer_p1b_log_ports))
            # acceptor -> p2b_proxy_leader, p2b
            acceptor_programs[acceptor_id][i].ports.p2b.send_to(hydro.demux(to_p2b_proxy_leader_ports))
        # coordinator -> acceptor, p1a_commit
        coordinator_programs[acceptor_id].ports.p1a_commit.send_to(hydro.demux(acceptor_p1a_commit_ports))
    for proposer_id in range(0, f+1):
        for i in range(0, num_p2a_proxy_leaders):
            # p2a_proxy_leader -> acceptor, p2a
            p2a_proxy_leader_programs[proposer_id][i].ports.p2a_from_proxy.send_to(hydro.demux(acceptor_p2a_ports))
        for i in range(0, num_p2b_proxy_leaders):
            # p2b_proxy_leader -> proposer, p2b
            p2b_proxy_leader_programs[proposer_id][i].ports.p2b_to_proposer.send_to(hydro.demux(proposer_p2b_ports))
            # p2b_proxy_leader -> proposer, inputs
            p2b_proxy_leader_programs[proposer_id][i].ports.inputs.send_to(hydro.demux(proposer_inputs_ports))
        

    await deployment.deploy()

    print("deployed!")

    # create this as separate variable to indicate to Hydro that we want to capture all stdout, even after the loop
    program_out = await proposer_programs[0].stdout()

    await deployment.start()
    print("started!")

    counter = 0
    async for log in program_out:
        print(log)
        counter += 1
        if counter == 1000:
            break

    print(await proposer_programs[0].exit_code())

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
