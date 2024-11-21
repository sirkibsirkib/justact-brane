use std::collections::HashMap;

use slick::{GroundAtom, Program as Policy};

type MessageId = GroundAtom;
type AgentId = GroundAtom;
type Variable = GroundAtom;
type Time = u64;
type Signature = u128;

type AssetData = Vec<u8>;

#[derive(Debug, Clone)]
struct Message {
    message_id: MessageId,
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

#[derive(Default)]
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
    AllAgents,
    ListedAgents(Vec<AgentId>),
}
struct Inform {
    info: Info,
    recipients: Recipients,
}

type AgentDo = Box<dyn Fn(&AgentState) -> Vec<Inform>>;
struct AgentState {
    agent_local: Store,
    agent_do: AgentDo,
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
    current_time: Time,
    consortium_do: ConsortiumDo,
}

impl System {
    fn handle_inform(&mut self, Inform { info, recipients }: Inform) {
        let all_agent_ids: Vec<AgentId> = self.agents.keys().cloned().collect();
        let recipients_ids = match recipients {
            Recipients::AllAgents => all_agent_ids.clone(),
            Recipients::ListedAgents(v) => v.clone(), // Recipients::Specific(agent_ids) => agent_ids.iter(),
        };
        for recipient_id in recipients_ids {
            if let Some(store) = self.agents.get_mut(&recipient_id) {
                match &info {
                    Info::GossipStated(s) => store.agent_local.stated.push(s.clone()),
                    Info::GossipEnacted(e) => store.agent_local.enacted.push(e.clone()),
                }
            }
        }
    }
    fn handle_consortium_deed(&mut self, deed: ConsortiumDeed) {
        match deed {
            ConsortiumDeed::Agree(a) => self.agreed.push(a),
            ConsortiumDeed::ChangeCurrentTime(t) => self.current_time = t,
            ConsortiumDeed::EndScenario => return,
        }
    }
    fn run(&mut self) {
        // let read_pattern: Pattern = slick::parse::atom("Agent reads Variable").unwrap().1;
        // let write_pattern: Fact = slick::parse::atom("Agent reads Variable").unwrap().1;
        let mut inform_buffer: Vec<Inform> = vec![];
        loop {
            // phase 1: handle synchronous comms
            for deed in (self.consortium_do)(&self) {
                self.handle_consortium_deed(deed)
            }

            // phase 2: handle agent-local comms
            for agent_state in self.agents.values() {
                inform_buffer.extend((agent_state.agent_do)(&agent_state));
            }
            for inform in inform_buffer.drain(..) {
                self.handle_inform(inform)
            }
        }
    }
}

static CONSORTIUM_1_SRC: &str = "";

struct Keypair {
    public: u64,
    private: u64,
}
struct AgentInit {
    agent_do: AgentDo,
    keypair: Keypair,
}

fn scenario(agent_inits: HashMap<AgentId, AgentInit>, consortium_do: ConsortiumDo) {
    let mut system = System {
        consortium_do,
        current_time: 1,
        agents: agent_inits
            .into_iter()
            .map(|(id, agent_init)| {
                (
                    id,
                    AgentState {
                        agent_do: agent_init.agent_do,
                        agent_local: Default::default(),
                    },
                )
            })
            .collect(),
        agreed: Default::default(),
    };
    let consortium_1_message = Message {
        message_id: slick::parse::atom("consortium 1")
            .unwrap()
            .1
            .try_as_ground_atom()
            .unwrap()
            .clone(),
        payload: slick::parse::program(CONSORTIUM_1_SRC)
            .expect("consortium 1 parse fail")
            .1,
    };
    // let consortium_1_statement = system.handle_inform(Inform {
    //     recipients: Recipients::AllAgents,
    //     info: Info::GossipStated(),
    // });
}
