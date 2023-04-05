use std::time::{SystemTime, UNIX_EPOCH};
use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow::bytes::BytesMut;
use hydroflow::tokio_stream::wrappers::IntervalStream;
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let p1a_port = ports
        .remove("p1a")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let acceptors = p1a_port.keys.clone();
    println!("acceptors: {:?}", acceptors);
    let p1a_sink = p1a_port.into_sink();

    let p1b_source = ports
        .remove("p1b")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let p1b_log_source = ports
        .remove("p1b_log")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let p2a_sink = ports
        .remove("p2a")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let p2b_source = ports
        .remove("p2b")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let i_am_leader_port = ports
        .remove("i_am_leader_sink")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let proposers = i_am_leader_port.keys.clone();
    println!("proposers: {:?}", proposers);

    let i_am_leader_sink = i_am_leader_port.into_sink();

    let i_am_leader_source = ports
        .remove("i_am_leader_source")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let (my_id, quorum, heartbeat_timeout_const, p1a_timeout_const):(u32, u32, u32, u32) = 
        serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    let p1a_timeout = IntervalStream::new(tokio::time::interval(std::time::Duration::from_secs(p1a_timeout_const.into())));
    
    let mut df = datalog!(
        r#"

######################## relation definitions
# EDB
.input id `repeat_iter([(my_id,),])`
.input acceptors `repeat_iter(acceptors.clone()) -> map(|p| (p,))`
.input proposers `repeat_iter(proposers.clone()) -> map(|p| (p,))`
.input quorum `repeat_iter([(quorum,),])`
.input leaderTimeout `repeat_iter([(heartbeat_timeout_const,),])` # number of seconds past which a leader is considered expired
.input noop `repeat_iter([("noop".to_string(),),])`

# Debug
.output p1aOut `for_each(|(a,pid,id,num):(u32,u32,u32,u32,)| println!("p1a sent from {:?} to {:?}: [{:?},{:?},{:?}]", pid, a, pid, id, num))`
.output p1bOut `for_each(|(pid,a,log_size,id,num,max_id,max_num):(u32,u32,u32,u32,u32,u32,u32,)| println!("p1b received at {:?}: [{:?},{:?},{:?},{:?},{:?},{:?}]", pid, a, log_size, id, num, max_id, max_num))`
.output p1bLogOut `for_each(|(pid,a,payload,slot,payload_id,payload_num,id,num):(u32,u32,String,u32,u32,u32,u32,u32,)| println!("p1bLog received at {:?}: [{:?},{:?},{:?},{:?},{:?},{:?},{:?}]", pid, a, payload, slot, payload_id, payload_num, id, num))`
.output p2aOut `for_each(|(a,pid,payload,slot,id,num):(u32,u32,String,u32,u32,u32,)| println!("p2a sent from {:?} to {:?}: [{:?},{:?},{:?},{:?},{:?}]", pid, a, pid, payload, slot, id, num))`
.output p2bOut `for_each(|(pid,a,payload,slot,id,num,max_id,max_num):(u32,u32,String,u32,u32,u32,u32,u32,)| println!("p2b received at {:?}: [{:?},{:?},{:?},{:?},{:?},{:?},{:?}]]", pid, a, payload, slot, id, num, max_id, max_num))`
.output iAmLeaderSendOut `for_each(|(dest,pid,num):(u32,u32,u32,)| println!("iAmLeader sent from {:?} to {:?}: [{:?},{:?}]", pid, dest, pid, num))`
.output iAmLeaderReceiveOut `for_each(|(my_id,pid,num):(u32,u32,u32,)| println!("iAmLeader received at {:?}: [{:?},{:?}]", my_id, pid, num))`

# IDB
.input clientIn `repeat_iter([("vote".to_string(),),])`
.output clientOut `for_each(|(payload,slot):(String,u32)| println!("committed {:?}: {:?}", slot, payload))`

.input startBallot `repeat_iter([(0 as u32,),])`

# p1a: proposerID, ballotID, ballotNum
.async p1a `map(|(node_id, v):(u32,(u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p1a_sink)` `null::<(u32,u32,u32)>()` 
# p1b: acceptorID, logSize, ballotID, ballotNum, maxBallotID, maxBallotNum
.async p1bU `null::<(u32,u32,u32,u32,u32,u32,)>()` `source_stream(p1b_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32,u32,u32,u32,)>(v.unwrap()).unwrap())`
# p1bLog: acceptorID, payload, slot, payloadBallotID, payloadBallotNum, ballotID, ballotNum
.async p1bLogU `null::<(u32,String,u32,u32,u32,u32,u32,)>()` `source_stream(p1b_log_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,String,u32,u32,u32,u32,u32,)>(v.unwrap()).unwrap())`
# p2a: proposerID, payload, slot, ballotID, ballotNum
.async p2a `map(|(node_id, v):(u32,(u32,String,u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p2a_sink)` `null::<(u32,String,u32,u32,u32)>()` 
# p2b: acceptorID, payload, slot, ballotID, ballotNum, maxBallotID, maxBallotNum
.async p2bU `null::<(u32,String,u32,u32,u32,u32,u32)>()` `source_stream(p2b_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,String,u32,u32,u32,u32,u32,)>(v.unwrap()).unwrap())`

.input timeout `source_stream(p1a_timeout) -> map(|_| () )` # periodic timer to send p1a
.input currTime `repeat_iter(vec![()]) -> map(|_| (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32,))` # wall-clock time
# iAmLeader: ballotID, ballotNum. Note: this is both a source and a sink
.async iAmLeaderU `map(|(node_id, v):(u32,(u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(i_am_leader_sink)` `source_stream(i_am_leader_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,)>(v.unwrap()).unwrap())`
######################## end relation definitions


# inputs that are persisted must have an alias. Format: inputU = unpersisted input.
p1b(a, l, i, n, mi, mn) :- p1bU(a, l, i, n, mi, mn)
p1b(a, l, i, n, mi, mn) :+ p1b(a, l, i, n, mi, mn)
p1bLog(a, p, s, pi, pn, i, n) :- p1bLogU(a, p, s, pi, pn, i, n)
p1bLog(a, p, s, pi, pn, i, n) :+ p1bLog(a, p, s, pi, pn, i, n)
p2b(a, p, s, i, n, mi, mn) :- p2bU(a, p, s, i, n, mi, mn)
p2b(a, p, s, i, n, mi, mn) :+ p2b(a, p, s, i, n, mi, mn)
receivedBallots(i, n) :+ receivedBallots(i, n)
payloads(p) :- clientIn(p)
payloads(p) :+ payloads(p)
iAmLeader(i, n, arrivalTime) :- iAmLeaderU(i, n), currTime(arrivalTime)
iAmLeader(i, n, arrivalTime) :+ iAmLeader(i, n, arrivalTime)

# Initialize
ballot(zero) :- startBallot(zero)
iAmLeader(zero, zero, zero) :- startBallot(zero)

# Debug
p1aOut(a, i, i, num) :- id(i), NewBallot(num), timeout(), LeaderExpired(), acceptors(a)
p1aOut(a, i, i, num) :- id(i), ballot(num), !NewBallot(newNum), timeout(), LeaderExpired(), acceptors(a)
p1bOut(pid, a, logSize, id, num, maxID, maxNum) :- id(pid), p1bU(a, logSize, id, num, maxID, maxNum)
p1bLogOut(pid, a, payload, slot, payloadBallotID, payloadBallotNum, id, num) :- id(pid), p1bLogU(a, payload, slot, payloadBallotID, payloadBallotNum, id, num)
p2aOut(a, i, payload, slot, i, num) :- id(pid), ResentEntries(payload, slot), id(i), ballot(num), acceptors(a)
p2aOut(a, i, no, slot, i, num) :- FilledHoles(no, slot), id(i), ballot(num), acceptors(a)
p2aOut(a, i, payload, slot + 1, i, num) :- ChosenPayload(payload), MaxProposedSlot(slot), id(i), ballot(num), acceptors(a)
iAmLeaderSendOut(pid, i, num) :- id(i), ballot(num), IsLeader(), proposers(pid), !id(pid)
iAmLeaderReceiveOut(pid, i, num) :- id(pid), iAmLeaderU(i, num)


######################## stable leader election
RelevantP1bs(acceptorID, logSize) :- p1b(acceptorID, logSize, i, num, maxID, maxNum), id(i), ballot(num)
CountMatchingP1bs(count(acceptorID)) :- RelevantP1bs(acceptorID, logSize)
receivedBallots(id, num) :- iAmLeader(id, num, arrivalTime)
receivedBallots(maxBallotID, maxBallotNum) :- p1b(acceptorID, logSize, i, num, maxBallotID, maxBallotNum)
receivedBallots(maxBallotID, maxBallotNum) :- p2b(acceptorID, payload, slot, ballotID, ballotNum, maxBallotID, maxBallotNum)
MaxReceivedBallotNum(max(num)) :- receivedBallots(id, num)
MaxReceivedBallot(max(id), num) :- MaxReceivedBallotNum(num), receivedBallots(id, num)
IsLeader() :- CountMatchingP1bs(c), quorum(size), (c >= size), id(i), ballot(num), MaxReceivedBallot(maxId, maxNum), (num > maxNum)
IsLeader() :- CountMatchingP1bs(c), quorum(size), (c >= size), id(i), ballot(num), MaxReceivedBallot(maxId, maxNum), (num == maxNum), (i > maxId)

# send heartbeat if we're the leader.
iAmLeaderU@pid(i, num) :~ id(i), ballot(num), IsLeader(), proposers(pid), !id(pid) # don't send to self
LatestIAmLeader(max(arrivalTime)) :- iAmLeader(i, num, arrivalTime)
TimeBetweenHeartbeat(t - arrivalTime) :- LatestIAmLeader(arrivalTime), currTime(t)
LeaderExpired() :- TimeBetweenHeartbeat(t), leaderTimeout(timeout), (t > timeout), !IsLeader()


# Resend p1a if we waited a random amount of time (timeout) AND leader heartbeat timed out. Send NewBallot if it was just triggered (ballot is updated in t+1), otherwise send ballot.
p1a@a(i, i, num) :~ id(i), NewBallot(num), timeout(), LeaderExpired(), acceptors(a)
p1a@a(i, i, num) :~ id(i), ballot(num), !NewBallot(newNum), timeout(), LeaderExpired(), acceptors(a)

# ballot = max + 1. If anothe proposer sends iAmLeader, that contains its ballot, which updates our ballot (to be even higher), so we are no longer the leader (RelevantP1bs no longer relevant)
NewBallot(maxNum + 1) :- MaxReceivedBallot(maxId, maxNum), id(i), ballot(num), (maxNum >= num), (maxId != i)
ballot(num) :+ NewBallot(num)
ballot(num) :+ ballot(num), !NewBallot(newNum)
######################## end stable leader election 



######################## reconcile p1b log with local log
RelevantP1bLogs(acceptorID, payload, slot, payloadBallotID, payloadBallotNum) :- p1bLog(acceptorID, payload, slot, payloadBallotID, payloadBallotNum, i, num), id(i), ballot(num)

# cannot send new p2as until all p1b acceptor logs are PROCESSED; otherwise might miss pre-existing entry
P1bLogFromAcceptor(acceptorID, count(slot)) :- RelevantP1bLogs(acceptorID, payload, slot, payloadBallotID, payloadBallotNum)
P1bAcceptorLogReceived(acceptorID) :- P1bLogFromAcceptor(acceptorID, logSize), RelevantP1bs(acceptorID, logSize)
P1bAcceptorLogReceived(acceptorID) :- RelevantP1bs(acceptorID, logSize), (logSize == 0)
P1bNumAcceptorsLogReceived(count(acceptorID)) :- P1bAcceptorLogReceived(acceptorID)
P1bQuorumReached() :- P1bNumAcceptorsLogReceived(c), quorum(size), (c >= size), IsLeader()
# logs don't count as all received until NEXT TIMESTEP; all p1b resends should've happened before then. If IsLeader is false, this is reset. In the timestep after that, we propose with a new ballot, so no quorum can be reached with p1b logs, until we actually win phase 1 again & receive all logs.
canSendP2a() :+ P1bQuorumReached()

P1bMatchingEntry(payload, slot, count(acceptorID), payloadBallotID, payloadBallotNum) :-  RelevantP1bLogs(acceptorID, payload, slot, payloadBallotID, payloadBallotNum)
# what was committed = store in local log. Note: Don't need to worry about overwriting; it's impossible to have f+1 matching for the same slot and another payload with a higher ballot; therefore this slot must already have the same payload (maybe with a lower ballot)
proposedLog(payload, slot) :- P1bMatchingEntry(payload, slot, c, payloadBallotID, payloadBallotNum), quorum(size), (c >= size)

# what was not committed = find max ballot, store in local log, resend 
P1bLargestEntryBallotNum(slot, max(payloadBallotNum)) :- RelevantP1bLogs(acceptorID, payload, slot, payloadBallotID, payloadBallotNum)
P1bLargestEntryBallot(slot, max(payloadBallotID), payloadBallotNum) :- P1bLargestEntryBallotNum(slot, payloadBallotNum), RelevantP1bLogs(acceptorID, payload, slot, payloadBallotID, payloadBallotNum)
# does not explicitly avoid resending committed entries, since proposedLog is negated, which means that committed entries (which are written to proposedLog in the same timestep) must be in an earlier strata and are implicitly avoided
# makes sure that p2as cannot be sent yet; otherwise resent slots might conflict. Once p2as can be sent, a new p1b log might tell us to propose a payload for the same slot we propose (in parallel) for p2a, which violates an invariant.
ResentEntries(payload, slot) :- P1bLargestEntryBallot(slot, payloadBallotID, payloadBallotNum), P1bMatchingEntry(payload, slot, c, payloadBallotID, payloadBallotNum), !proposedLog(otherPayload, slot), P1bQuorumReached(), !canSendP2a()
proposedLog(payload, slot) :+ ResentEntries(payload, slot) # must be succ because proposedLog is negated in ResentEntries
p2a@a(i, payload, slot, i, num) :~ ResentEntries(payload, slot), id(i), ballot(num), acceptors(a)

# hole filling: if a slot is not in ResentEntries or proposedLog but it's smaller than max, then propose noop. Provides invariant that all holes are filled (proposed) by next timestep and we can just assign slots as current slot+1
ProposedSlots(slot) :- proposedLog(payload, slot)
ProposedSlots(slot) :- ResentEntries(payload, slot) # ProposedSlots required in addition to proposedLog because ResentEntries' slots are not added to proposedLog until next timestep
MaxProposedSlot(max(slot)) :- ProposedSlots(slot)
PrevSlots(s) :- MaxProposedSlot(maxSlot), less_than(s, maxSlot)
FilledHoles(no, s) :- noop(no), !ProposedSlots(s), PrevSlots(s), P1bQuorumReached(), !canSendP2a()
proposedLog(no, s) :+ FilledHoles(no, s)
p2a@a(i, no, slot, i, num) :~ FilledHoles(no, slot), id(i), ballot(num), acceptors(a)

# only persist proposedLog if we're the leader. This way, when we lose election, the proposals are refreshed based on p1bs
proposedLog(p, s) :+ proposedLog(p, s), IsLeader()
######################## end reconcile p1b log with local log



######################## send p2as 
# assign a slot
ChosenPayload(choose(payload)) :- payloads(payload), !proposedLog(payload, slot), canSendP2a()
p2a@a(i, payload, slot + 1, i, num) :~ ChosenPayload(payload), MaxProposedSlot(slot), id(i), ballot(num), acceptors(a)
proposedLog(payload, slot + 1) :+ ChosenPayload(payload), MaxProposedSlot(slot)
######################## end send p2as 



######################## process p2bs
CountMatchingP2bs(payload, slot, count(acceptorID), i, num) :- p2b(acceptorID, payload, slot, i, num, payloadBallotID, payloadBallotNum)
client_out(payload, slot) :- CountMatchingP2bs(payload, slot, c, i, num), quorum(size), (c >= size)
######################## end process p2bs    
"#
    );

    df.run_async().await;
}
