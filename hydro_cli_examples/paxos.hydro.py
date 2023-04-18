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
    f = int(args[2])
    p1a_node_0_timeout = int(args[3])
    p1a_other_nodes_timeout = int(args[4])
    i_am_leader_resend_timeout = int(args[5])
    i_am_leader_check_timeout = int(args[6])
    inputs_per_tick = int(args[7])
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
    proposer_i_am_leader_ports = {}
    for i in range(0, f+1):
        proposer_i_am_leader_ports[i] = {}
    for i in range(0, f+1):
        machine1 = gcp_machine(deployment=deployment, gcp_vpc=gcp_vpc) if machine_1_gcp else localhost_machine
        proposer_machine.append(machine1)
        p1a_timeout = p1a_node_0_timeout if i == 0 else p1a_other_nodes_timeout # proposer with id 0 is much more likely to be the leader

        proposer = deployment.HydroflowCrate(
            src=".",
            example="dedalus_paxos_proposer",
            args=[json.dumps((i, f, p1a_timeout, i_am_leader_resend_timeout, i_am_leader_check_timeout, inputs_per_tick))], # my_id, f, p1a_timeout_const, i_am_leader_resend_timeout_const, i_am_leader_check_timeout_const, inputs_per_tick
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
        machine2 = gcp_machine(deployment=deployment, gcp_vpc=gcp_vpc) if machine_2_gcp else localhost_machine
        acceptor_machines.append(machine2)
        
        acceptor = deployment.HydroflowCrate(
            src=".",
            example="dedalus_paxos_acceptor",
            args=[json.dumps([i])], # my_id
            on=machine2
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
    program_out = await proposer_programs[0].stderr()

    await deployment.start()
    print("started!")

    # per_acceptor = [[] for _ in acceptor_programs]
    total_throughput = []
    # total_sent = []

    # fig = plt.figure()
    # ax = fig.add_subplot(1,1,1)
    plt.ion()               # interactive mode on
    fig,ax = plt.subplots()

    # acceptor_plots = []
    # for i in range(0, len(per_acceptor)):
    #     acceptor_plots.append(ax.plot(range(0, len(per_acceptor[i])), per_acceptor[i], label="acceptor " + str(i))[0])
    total_throughput_plot = ax.plot(range(0, len(total_throughput)), total_throughput, label="total throughput")[0]
    # total_sent_plot = ax.plot(range(0, len(total_sent)), total_sent, label="total sent")[0]
    plt.legend()
    plt.xlabel("index in array")
    plt.ylabel("throughput")
    fig.show()

    try:
        async for log in program_out:
            split = log.split(",")
            # if split[0] == "throughput":
            #     per_acceptor[int(split[1])].append(int(split[2]))
            if split[0] == "total_throughput":
                total_throughput.append(int(split[1]))
            # if split[0] == "total_sent":
            #     total_sent.append(int(split[1]))
            print(log, file=sys.stderr)


            # for i in range(0, len(per_acceptor)):
            #     acceptor_plots[i].set_xdata(range(0, len(per_acceptor[i])))
            #     acceptor_plots[i].set_ydata(per_acceptor[i])
            
            throughput_seconds = range(0, len(total_throughput))
            total_throughput_plot.set_xdata(throughput_seconds)
            total_throughput_plot.set_ydata(total_throughput)

            # sent_seconds = range(0, len(total_sent))
            # total_sent_plot.set_xdata(sent_seconds)
            # total_sent_plot.set_ydata(total_sent)

            # Calculate slope
            if len(total_throughput) > 1:
                throughput_z = np.polyfit(throughput_seconds, total_throughput, 1)
                # sent_z = np.polyfit(sent_seconds, total_sent, 1)
                for x in plt.findobj(match=text.Text):
                    try:
                        x.remove()
                    except NotImplementedError:
                        pass
                # plt.text(0.7, 0.1, "throughput=%.2f/s"%throughput_z[0] + "\nwrite=%.2f/s"%sent_z[0], fontsize = 11, transform=ax.transAxes, horizontalalignment='right', verticalalignment='bottom')
                plt.text(0.7, 0.1, "throughput=%.2f/s"%throughput_z[0], fontsize = 11, transform=ax.transAxes, horizontalalignment='right', verticalalignment='bottom')


            ax.relim()
            ax.autoscale_view()
            plt.draw()
            plt.pause(0.01)
    except:
        import traceback
        traceback.print_exc(file=sys.stderr)
        pass

    # plot a line chart with one line for each acceptor and the x axis being indices in the array for that acceptor
    
    # plt.show()

    print(await proposer_programs[0].exit_code())

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
