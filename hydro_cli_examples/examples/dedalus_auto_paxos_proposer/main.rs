use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes, batched_sink
};
use hydroflow::bytes::BytesMut;
use hydroflow::tokio_stream::wrappers::IntervalStream;
use hydroflow_datalog::datalog;
use tokio::time::{interval_at, Duration, Instant};

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

    let p2a_port = ports
        .remove("p2a")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let p2a_proxy_leaders = p2a_port.keys.clone();
    println!("p2a_proxy_leaders: {:?}", p2a_proxy_leaders);
    let p2a_sink = p2a_port.into_sink();

    let p2b_source = ports
        .remove("p2b")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let inputs_source = ports
        .remove("inputs")
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

    let (my_id, f, num_acceptor_groups, num_p2a_proxy_leaders, p1a_timeout_const, i_am_leader_resend_timeout_const, i_am_leader_check_timeout_const, flush_every_n):(u32, u32, u32, u32, u32, u32, u32, usize) = 
        serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    let p2a_proxy_leaders_start_id = my_id * num_p2a_proxy_leaders;
    let p1a_timeout = periodic(p1a_timeout_const);
    let i_am_leader_resend_timeout = periodic(i_am_leader_resend_timeout_const);
    let i_am_leader_check_timeout = periodic(i_am_leader_check_timeout_const);

    // let p2a_sink = batched_sink(p2a_unbatched_sink, flush_every_n, Duration::from_secs(10));

    let mut df = datalog!(
        r#"
        ######################## relation definitions
# EDB
.input id `repeat_iter([(my_id,),])`
.input numAcceptorGroups `repeat_iter([(num_acceptor_groups,),])`
.input acceptors `repeat_iter(acceptors.clone()) -> map(|p| (p,))`
.input proposers `repeat_iter(proposers.clone()) -> map(|p| (p,))`
.input p2aProxyLeadersStartID `repeat_iter([(p2a_proxy_leaders_start_id,),])`
.input numP2aProxyLeaders `repeat_iter([(num_p2a_proxy_leaders,),])` # ID scheme: Assuming num_p2a_proxy_leaders = n (per proposer). Proposer i has proxy leaders from i*n to (i+1)*n-1
.input quorum `repeat_iter([(f+1,),])`
.input noop `repeat_iter([(0 as u32,),])`

# Debug
.output p1aOut `for_each(|(a,pid,id,num):(u32,u32,u32,u32,)| println!("proposer {:?} sent p1a to acceptor {:?}: [{:?},{:?},{:?}]", pid, a, pid, id, num))`
.output p1bOut `for_each(|(pid,p,a,log_size,id,num,max_id,max_num):(u32,u32,u32,u32,u32,u32,u32,u32,)| println!("proposer {:?} received p1b from acceptor: [{:?},{:?},{:?},{:?},{:?},{:?},{:?}]", pid, p, a, log_size, id, num, max_id, max_num))`
.output p1bLogOut `for_each(|(pid,p,a,payload,slot,payload_id,payload_num,id,num):(u32,u32,u32,u32,u32,u32,u32,u32,u32,)| println!("proposer {:?} received p1bLog from acceptor: [{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?}]", pid, p, a, payload, slot, payload_id, payload_num, id, num))`
.output p2aOut `for_each(|(a,pid,payload,slot,id,num):(u32,u32,u32,u32,u32,u32,)| println!("proposer {:?} sent p2a to p2aProxyLeader {:?}: [{:?},{:?},{:?},{:?},{:?}]", pid, a, pid, payload, slot, id, num))`
.output p2bOut `for_each(|(pid,max_id,max_num,t1):(u32,u32,u32,u32,)| println!("proposer {:?} received p2b from p2bProxyLeader: [{:?},{:?},{:?}]", pid, max_id, max_num, t1))`
.output p2bSealedOut `for_each(|(pid,max_id,max_num):(u32,u32,u32,)| println!("proposer {:?} sealed p2b: [{:?},{:?}]", pid, max_id, max_num))`
.output inputsOut `for_each(|(pid,n,t1,prev_t):(u32,u32,u32,u32,)| println!("proposer {:?} received inputs from p2bProxyLeader: [{:?},{:?},{:?}]", pid, n, t1, prev_t))`
.output iAmLeaderSendOut `for_each(|(dest,pid,num):(u32,u32,u32,)| println!("proposer {:?} sent iAmLeader to proposer{:?}: [{:?},{:?}]", pid, dest, pid, num))`
.output iAmLeaderReceiveOut `for_each(|(my_id,pid,num):(u32,u32,u32,)| println!("proposer {:?} received iAmLeader from proposer: [{:?},{:?}]", my_id, pid, num))`
# For some reason Hydroflow can't infer the type of nextSlot, so we define it manually:
.input nextSlot `null::<(u32,)>()`

# IDB
.input clientIn `repeat_iter_external(vec![()]) -> map(|_| (context.current_tick() as u32,))`
.output clientOut `for_each(|(payload,slot):(u32,u32)| println!("committed {:?}: {:?}", slot, payload))`

.input startBallot `repeat_iter([(0 as u32,),])`
.input startSlot `repeat_iter([(0 as u32,),])`

# p1a: proposerID, ballotID, ballotNum
.async p1a `map(|(node_id, v):(u32,(u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p1a_sink)` `null::<(u32,u32,u32)>()` 
# p1b: partitionID, acceptorID, logSize, ballotID, ballotNum, maxBallotID, maxBallotNum
.async p1bU `null::<(u32,u32,u32,u32,u32,u32,u32,)>()` `source_stream(p1b_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32,u32,u32,u32,u32,)>(v.unwrap()).unwrap())`
# p1bLog: partitionID, acceptorID, payload, slot, payloadBallotID, payloadBallotNum, ballotID, ballotNum
.async p1bLogU `null::<(u32,u32,u32,u32,u32,u32,u32,u32,)>()` `source_stream(p1b_log_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32,u32,u32,u32,u32,u32,)>(v.unwrap()).unwrap())`
# p2a: proposerID, payload, slot, ballotID, ballotNum
.async p2a `map(|(node_id, v):(u32,(u32,u32,u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p2a_sink)` `null::<(u32,u32,u32,u32,u32)>()` 
# p2bU: maxBallotID, maxBallotNum, t1
.async p2bU `null::<(u32,u32,u32)>()` `source_stream(p2b_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32)>(v.unwrap()).unwrap())`
# inputs: n, t1, prevT
.async inputsU `null::<(u32,u32,u32)>()` `source_stream(inputs_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32)>(v.unwrap()).unwrap())`

.input p1aTimeout `source_stream(p1a_timeout) -> map(|_| () )` # periodic timer to send p1a, so proposers each send at random times to avoid contention
.input iAmLeaderResendTimeout `source_stream(i_am_leader_resend_timeout) -> map(|_| () )` # periodic timer to resend iAmLeader
.input iAmLeaderCheckTimeout `source_stream(i_am_leader_check_timeout) -> map(|_| () )` # periodic timer to check if the leader has sent a heartbeat
// .input currTime `repeat_iter(vec![()]) -> map(|_| (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32,))` # wall-clock time
# iAmLeader: ballotID, ballotNum. Note: this is both a source and a sink
.async iAmLeaderU `map(|(node_id, v):(u32,(u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(i_am_leader_sink)` `source_stream(i_am_leader_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,)>(v.unwrap()).unwrap())`
######################## end relation definitions


# inputs that are persisted must have an alias. Format: inputU = unpersisted input.
p1b(p, a, l, i, n, mi, mn) :- p1bU(p, a, l, i, n, mi, mn)
p1b(p, a, l, i, n, mi, mn) :+ p1b(p, a, l, i, n, mi, mn)
// .persist p1b
p1bLog(p, a, payload, s, pi, pn, i, n) :- p1bLogU(p, a, payload, s, pi, pn, i, n)
p1bLog(p, a, payload, s, pi, pn, i, n) :+ p1bLog(p, a, payload, s, pi, pn, i, n)
// .persist p1bLog
receivedBallots(i, n) :+ receivedBallots(i, n)
// .persist receivedBallots
iAmLeader(i, n) :- iAmLeaderU(i, n)
iAmLeader(i, n) :+ iAmLeader(i, n), !iAmLeaderCheckTimeout() # clear iAmLeader periodically (like LRU clocks)

# Initialize
ballot(zero) :- startBallot(zero)

# Debug
p1aOut(a, i, i, num) :- id(i), NewBallot(num), p1aTimeout(), LeaderExpired(), acceptors(a)
p1aOut(a, i, i, num) :- id(i), ballot(num), !NewBallot(newNum), p1aTimeout(), LeaderExpired(), acceptors(a)
p1bOut(pid, p, a, logSize, id, num, maxID, maxNum) :- id(pid), p1bU(p, a, logSize, id, num, maxID, maxNum)
p1bLogOut(pid, p, a, payload, slot, payloadBallotID, payloadBallotNum, id, num) :- id(pid), p1bLogU(p, a, payload, slot, payloadBallotID, payloadBallotNum, id, num)
// p2aOut(start+(slot%n), i, payload, slot, i, num) :- ResentLog(payload, slot), id(i), ballot(num), numP2aProxyLeaders(n), p2aProxyLeadersStartID(start)
p2aOut(start+(slot%n), i, no, slot, i, num) :- FilledHoles(no, slot), id(i), ballot(num), numP2aProxyLeaders(n), p2aProxyLeadersStartID(start) # Weird bug where if this line is commented out, id has an error?
// p2aOut(start+(slot%n), i, payload, slot, i, num) :- ChosenPayload(payload), nextSlot(slot), id(i), ballot(num),  numP2aProxyLeaders(n), p2aProxyLeadersStartID(start)
p2bOut(pid, mi, mn, t1) :- id(pid), p2bU(mi, mn, t1)
p2bSealedOut(pid, mi, mn) :- id(pid), p2bSealed(mi, mn)
inputsOut(pid, n, t1, prevT) :- id(pid), inputsU(n, t1, prevT)
// iAmLeaderSendOut(pid, i, num) :- id(i), ballot(num), IsLeader(), proposers(pid), iAmLeaderResendTimeout(), !id(pid) 
// iAmLeaderReceiveOut(pid, i, num) :- id(pid), iAmLeaderU(i, num)


######################## stable leader election
RelevantP1bs(partitionID, acceptorID, logSize) :- p1b(partitionID, acceptorID, logSize, i, num, maxID, maxNum), id(i), ballot(num)
receivedBallots(id, num) :- iAmLeader(id, num)
receivedBallots(maxBallotID, maxBallotNum) :- p1b(partitionID, acceptorID, logSize, i, num, maxBallotID, maxBallotNum)
receivedBallots(maxBallotID, maxBallotNum) :- p2bSealed(maxBallotID, maxBallotNum)
MaxReceivedBallotNum(max(num)) :- receivedBallots(id, num)
MaxReceivedBallot(max(id), num) :- MaxReceivedBallotNum(num), receivedBallots(id, num)
HasLargestBallot() :- MaxReceivedBallot(maxId, maxNum), id(i), ballot(num), (num > maxNum)
HasLargestBallot() :- MaxReceivedBallot(maxId, maxNum), id(i), ballot(num), (num == maxNum), (i >= maxId)

# send heartbeat if we're the leader.
iAmLeaderU@pid(i, num) :~ id(i), ballot(num), IsLeader(), proposers(pid), iAmLeaderResendTimeout(), !id(pid) # don't send to self
LeaderExpired() :- iAmLeaderCheckTimeout(), !iAmLeader(i, n), !IsLeader()

# Resend p1a if we waited a random amount of time (timeout) AND leader heartbeat timed out. Send NewBallot if it was just triggered (ballot is updated in t+1), otherwise send ballot.
p1a@a(i, i, num) :~ id(i), NewBallot(num), p1aTimeout(), LeaderExpired(), acceptors(a)
p1a@a(i, i, num) :~ id(i), ballot(num), !NewBallot(newNum), p1aTimeout(), LeaderExpired(), acceptors(a)

# ballot = max + 1. If anothe proposer sends iAmLeader, that contains its ballot, which updates our ballot (to be even higher), so we are no longer the leader (RelevantP1bs no longer relevant)
NewBallot(maxNum + 1) :- MaxReceivedBallot(maxId, maxNum), id(i), ballot(num), (maxNum >= num), (maxId != i)
ballot(num) :+ NewBallot(num)
ballot(num) :+ ballot(num), !NewBallot(newNum)
######################## end stable leader election 



######################## reconcile p1b log with local log
RelevantP1bLogs(partitionID, acceptorID, payload, slot, payloadBallotID, payloadBallotNum) :- p1bLog(partitionID, acceptorID, payload, slot, payloadBallotID, payloadBallotNum, i, num), id(i), ballot(num)

# cannot send new p2as until all p1b acceptor logs are PROCESSED; otherwise might miss pre-existing entry
P1bLogFromAcceptor(partitionID, acceptorID, count(slot)) :- RelevantP1bLogs(partitionID, acceptorID, payload, slot, payloadBallotID, payloadBallotNum)
P1bAcceptorLogReceived(partitionID, acceptorID) :- P1bLogFromAcceptor(partitionID, acceptorID, logSize), RelevantP1bs(partitionID, acceptorID, logSize)
P1bAcceptorLogReceived(partitionID, acceptorID) :- RelevantP1bs(partitionID, acceptorID, logSize), (logSize == 0)
P1bNumAcceptorPartitionsLogReceived(count(partitionID), acceptorID) :- P1bAcceptorLogReceived(partitionID, acceptorID)
// .output P1bNumAcceptorPartitionsLogReceived `for_each(|(c,aid):(u32,u32)| println!("P1bNumAcceptorPartitionsLogReceived: [{:?},{:?}]", c, aid))`
P1bNumAcceptorsLogReceived(count(acceptorID)) :- P1bNumAcceptorPartitionsLogReceived(n, acceptorID), numAcceptorGroups(n)
// .output P1bNumAcceptorsLogReceived `for_each(|(c,):(u32,)| println!("P1bNumAcceptorsLogReceived: [{:?}]", c))`
IsLeader() :- P1bNumAcceptorsLogReceived(c), quorum(size), (c >= size), HasLargestBallot()

P1bMatchingEntry(payload, slot, count(acceptorID), payloadBallotID, payloadBallotNum) :- RelevantP1bLogs(partitionID, acceptorID, payload, slot, payloadBallotID, payloadBallotNum)
# what was committed = store in local log. Note: Don't need to worry about overwriting; it's impossible to have f+1 matching for the same slot and another payload with a higher ballot; therefore this slot must already have the same payload (maybe with a lower ballot)
CommittedLog(payload, slot) :- P1bMatchingEntry(payload, slot, c, payloadBallotID, payloadBallotNum), quorum(size), (c >= size)

# what was not committed = find max ballot, store in local log, resend 
P1bLargestEntryBallotNum(slot, max(payloadBallotNum)) :- RelevantP1bLogs(partitionID, acceptorID, payload, slot, payloadBallotID, payloadBallotNum)
P1bLargestEntryBallot(slot, max(payloadBallotID), payloadBallotNum) :- P1bLargestEntryBallotNum(slot, payloadBallotNum), RelevantP1bLogs(partitionID, acceptorID, payload, slot, payloadBallotID, payloadBallotNum)
# makes sure that p2as cannot be sent yet; otherwise resent slots might conflict. Once p2as can be sent, a new p1b log might tell us to propose a payload for the same slot we propose (in parallel) for p2a, which violates an invariant.
ResentLog(payload, slot) :- P1bLargestEntryBallot(slot, payloadBallotID, payloadBallotNum), P1bMatchingEntry(payload, slot, c, payloadBallotID, payloadBallotNum), !CommittedLog(otherPayload, slot), IsLeader(), !nextSlot(s)
p2a@(start+(slot%n))(i, payload, slot, i, num) :~ ResentLog(payload, slot), id(i), ballot(num), numP2aProxyLeaders(n), p2aProxyLeadersStartID(start)

# hole filling: if a slot is not in ResentEntries or proposedLog but it's smaller than max, then propose noop. Provides invariant that all holes are filled (proposed) by next timestep and we can just assign slots as current slot+1
ProposedSlots(slot) :- startSlot(slot)
ProposedSlots(slot) :- CommittedLog(payload, slot)
ProposedSlots(slot) :- ResentLog(payload, slot)
MaxProposedSlot(max(slot)) :- ProposedSlots(slot)
PrevSlots(s) :- MaxProposedSlot(maxSlot), less_than(s, maxSlot)
FilledHoles(no, s) :- noop(no), !ProposedSlots(s), PrevSlots(s), IsLeader(), !nextSlot(s2)
p2a@(start+(slot%n))(i, no, slot, i, num) :~ FilledHoles(no, slot), id(i), ballot(num), numP2aProxyLeaders(n), p2aProxyLeadersStartID(start)

# To assign values sequential slots after reconciling p1bs, start at max+1
nextSlot(s+1) :+ IsLeader(), MaxProposedSlot(s), !nextSlot(s2)
######################## end reconcile p1b log with local log



######################## send p2as 
# assign a slot
ChosenPayload(choose(payload)) :- clientIn(payload), nextSlot(s), IsLeader() # drop all payloads that we can't handle in this tick by not persisting clientIn
p2a@(start+(slot%n))(i, payload, slot, i, num) :~ ChosenPayload(payload), nextSlot(slot), id(i), ballot(num), numP2aProxyLeaders(n), p2aProxyLeadersStartID(start)
# Increment the slot if a payload was chosen
nextSlot(s+1) :+ ChosenPayload(payload), nextSlot(s)
# Don't increment the slot if no payload was chosen, but we are still the leader
nextSlot(s) :+ !ChosenPayload(payload), nextSlot(s), IsLeader()
######################## end send p2as 



######################## p2bs with asymmetric decoupling
p2bUP(mi, mn, t1) :- p2bU(mi, mn, t1) # Accept the original input
p2bUP(mi, mn, t1) :+ p2bUP(mi, mn, t1) # Persist
recvSize(count(*), t1) :- p2bUP(mi, mn, t1) # Count. Since there's only 1 r, recvSize = rCount.
inputs(n, t1, prevT) :+ inputsU(n, t1, prevT)
inputs(n, t1, prevT) :+ inputs(n, t1, prevT) # TODO: experiment with replacing with .persist
canSeal(t1) :- recvSize(n, t1), inputs(n, t1, prevT), sealed(prevT), !sealed(t1) # Check if all inputs of this batch have been received
canSeal(t1) :- recvSize(n, t1), inputs(n, t1, prevT), !sealed(t2), (prevT == 0)
sealed(t1) :+ canSeal(t1)
sealed(t1) :+ sealed(t1) # TODO: experiment with replacing with .persist
p2bSealed(mi, mn) :- p2bUP(mi, mn, t1), canSeal(t1)
######################## end p2bs with asymmetric decoupling
"#
    );

    df.run_async().await;
}

fn periodic(timeout: u32) -> IntervalStream {
    let start = Instant::now() + Duration::from_secs(timeout.into());
    IntervalStream::new(interval_at(start, Duration::from_secs(timeout.into())))
}