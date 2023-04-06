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
    machine_2_gcp = args[1] == "gcp"
    f = int(args[2])
    heartbeat_timeout = int(args[3]) # in seconds
    leader_retry_node_0_timeout = int(args[4])
    leader_retry_other_nodes_timeout = int(args[5])
    i_am_leader_timeout = int(args[6])
    # Heartbeat timeout should >> I am leader timeout, so the current leader has time to send a heartbeat
    # Leader election time (out of our control) should >> leader retry timeout, so the leader doesn't spam acceptors. Leader retry timeout should differ between proposers to avoid contention

    deployment = hydro.Deployment()
    localhost_machine = deployment.Localhost()

    # set up proposer
    proposer_machine = []
    proposer_programs = []
    proposer_p1b_ports = {}
    proposer_p1b_log_ports = {}
    proposer_p2b_ports = {}
    proposer_i_am_leader_ports = {}
    for i in range(0, f+1):
        proposer_i_am_leader_ports[i] = {}
    for i in range(0, f+1):
        machine1 = gcp_machine(deployment=deployment) if machine_1_gcp else localhost_machine
        proposer_machine.append(machine1)
        leader_retry_timeout = leader_retry_node_0_timeout if i == 0 else leader_retry_other_nodes_timeout # proposer with id 0 is much more likely to be the leader

        proposer = deployment.HydroflowCrate(
            src=".",
            example="dedalus_paxos_proposer",
            args=[json.dumps((i, 2*f+1, heartbeat_timeout, leader_retry_timeout, i_am_leader_timeout))], # my_id, quorum, heartbeat_timeout_const, p1a_timeout_const, i_am_leader_timeout_const
            on=machine1
        )
        proposer_programs.append(proposer)
        proposer_p1b_ports[i] = proposer.ports.p1b.merge()
        proposer_p1b_log_ports[i] = proposer.ports.p1b_log.merge()
        proposer_p2b_ports[i] = proposer.ports.p2b.merge()
        for j in range(0, f+1):
            if i != j: # don't let proposers send to themselves
                proposer_i_am_leader_ports[j][i] = proposer.ports.i_am_leader_source.merge()

    # set up acceptors
    acceptor_machines = []
    acceptor_programs = []
    acceptor_p1a_ports = {}
    acceptor_p2a_ports = {}
    for i in range(0, 2*f+1):
        machine2 = gcp_machine(deployment=deployment) if machine_2_gcp else localhost_machine
        acceptor_machines.append(machine2)
        
        acceptor = deployment.HydroflowCrate(
            src=".",
            example="dedalus_paxos_acceptor",
            args=[json.dumps([i])], # my_id
            on=acceptor_machines[i]
        )
        acceptor.ports.p1b.send_to(hydro.demux(proposer_p1b_ports))
        acceptor.ports.p1b_log.send_to(hydro.demux(proposer_p1b_log_ports))
        acceptor.ports.p2b.send_to(hydro.demux(proposer_p2b_ports))
        acceptor_programs.append(acceptor)
        acceptor_p1a_ports[i] = acceptor.ports.p1a.merge()
        acceptor_p2a_ports[i] = acceptor.ports.p2a.merge()

    for i in range(0, f+1):
        proposer_programs[i].ports.p1a.send_to(hydro.demux(acceptor_p1a_ports))
        proposer_programs[i].ports.p2a.send_to(hydro.demux(acceptor_p2a_ports))
        proposer_programs[i].ports.i_am_leader_sink.send_to(hydro.demux(proposer_i_am_leader_ports[i]))

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
        if counter == 100:
            break

    print(await proposer_programs[0].exit_code())

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
