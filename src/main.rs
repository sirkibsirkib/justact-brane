use std::collections::HashMap;

use slick::{Atom as Fact, Program as Policy};

type MessageId = Fact;
type AgentId = Fact;
type Variable = Fact;
type Time = u64;
type Signature = u128;

type AssetData = Vec<u8>;

#[derive(Debug, Clone)]
struct Message {
    this_message: MessageId,
    author: AgentId,
    payload: Policy,
}

#[derive(Debug, Clone)]
struct Statement {
    msg: Message,
    author_signature: Signature,
}

#[derive(Debug)]
struct Agreement {
    content: Message,
    in_effect: Time,
}

#[derive(Debug, Clone)]
struct Action {
    basis: Message,
    enacts: Message,
    justification: Message,
    at: Time,
}

#[derive(Debug, Clone)]
struct Enacted {
    action: Action,
    actor_signature: Signature,
}

struct Store {
    stated: Vec<Statement>,
    enacted: Vec<Enacted>,
    assets: HashMap<Variable, AssetData>,
}

enum Info {
    GossipStated(Statement),
    GossipEnacted(Enacted),
}
enum Recipients {
    Broadcast,
    Specific(Vec<AgentId>),
}
struct Inform {
    info: Info,
    recipients: Recipients,
}

struct AgentState {
    agent_local: Store,
    agent_do: Box<dyn Fn(&AgentState) -> Vec<Inform>>,
}

enum ConsortiumDeed {
    Agree(Agreement),
    ChangeCurrentTime(Time),
    EndScenario,
}
type ConsortiumDo = Box<dyn Fn(&System) -> Vec<ConsortiumDeed>>;
struct System {
    agents: HashMap<AgentId, AgentState>,
    agreed: Vec<Agreement>,
    current: Time,
    consortium_do: ConsortiumDo,
}

impl System {
    fn run(&mut self) {
        let read_pattern: Fact = slick::parse::atom("Agent reads Variable").unwrap().1;
        let write_pattern: Fact = slick::parse::atom("Agent reads Variable").unwrap().1;
        let all_agent_ids: Vec<AgentId> = self.agents.keys().cloned().collect();
        let mut inform_buffer: Vec<Inform> = vec![];
        loop {
            // phase 1: handle synchronous comms
            for deed in (self.consortium_do)(&self) {
                match deed {
                    ConsortiumDeed::Agree(a) => self.agreed.push(a),
                    ConsortiumDeed::ChangeCurrentTime(t) => self.current = t,
                    ConsortiumDeed::EndScenario => return,
                }
            }

            // phase 2: handle agent-local comms
            for agent_state in self.agents.values() {
                inform_buffer.extend((agent_state.agent_do)(&agent_state));
            }
            for Inform { recipients, info } in inform_buffer.drain(..) {
                let recipients_ids = match recipients {
                    Recipients::Broadcast => all_agent_ids.iter(),
                    _ => unreachable!(), // Recipients::Specific(agent_ids) => agent_ids.iter(),
                };
                for recipient_id in recipients_ids {
                    if let Some(store) = self.agents.get_mut(recipient_id) {
                        match &info {
                            Info::GossipStated(s) => store.agent_local.stated.push(s.clone()),
                            Info::GossipEnacted(e) => store.agent_local.enacted.push(e.clone()),
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn main() {}
