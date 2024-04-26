use eyre::Result;
use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    futures::StreamExt,
    identity,
    mdns::{Mdns, MdnsEvent},
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    NetworkBehaviour, PeerId, Transport,
};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::{io::AsyncBufReadExt, sync::mpsc};

static KEY: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());
static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEY.public()));
static TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("Rustic Chain of Blocks"));

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TxInfo {
    from: String,
    to: String,
    value: u64,
    private_key: String,
}

enum EventType {
    Response(TxInfo),
    Input(String),
}

#[derive(NetworkBehaviour)]
struct TransactionBehaviour {
    floodsub: Floodsub,
    mdns: Mdns,
    #[allow(dead_code)]
    #[behaviour(ignore)]
    response_sender: mpsc::UnboundedSender<TxInfo>,
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for TransactionBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        match event {
            FloodsubEvent::Message(msg) => {
                if let Ok(tx_info) = serde_json::from_slice::<TxInfo>(&msg.data) {
                    info!(
                        "Received transaction: {:?} from peer {:?}",
                        tx_info, msg.source
                    );
                } else {
                    info!(
                        "Failed to deserialize message data from peer {:?}",
                        msg.source
                    );
                }
            }
            _ => (),
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for TransactionBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}
#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
    let auth_keys = Keypair::<X25519Spec>::new().into_authentic(&KEY)?;

    let transport = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let mut behaviour = TransactionBehaviour {
        floodsub: Floodsub::new(PEER_ID.clone()),
        mdns: Mdns::new(Default::default()).await?,
        response_sender,
    };

    behaviour.floodsub.subscribe(TOPIC.clone());

    let mut swarm = SwarmBuilder::new(transport, behaviour, PEER_ID.clone())
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        let evt = {
            tokio::select! {
                line = stdin.next_line() => Some(EventType::Input(line.expect("Input error!").expect("Input error!"))),
                response = response_rcv.recv() => Some(EventType::Response(response.expect("Response error!"))),
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Node started listening at {:?}", address);
                            None
                        },
                        SwarmEvent::ConnectionEstablished  { peer_id, .. } => {
                            info!("Connection established with peer {:?}", peer_id);
                            None
                        },
                        _ => None,
                    }
                },
            }
        };

        if let Some(event) = evt {
            match event {
                EventType::Response(resp) => {
                    let json = serde_json::to_string(&resp)?;
                    swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(TOPIC.clone(), json.as_bytes());
                }
                EventType::Input(line) => {
                    match line
                        .trim()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .as_slice()
                    {
                        ["tx", from, to, value, private_key] => {
                            let tx_info = TxInfo {
                                from: from.to_string(),
                                to: to.to_string(),
                                value: value.to_string().parse::<u64>()?,
                                private_key: private_key.to_string(),
                            };
                            let tx_info_json = serde_json::to_string(&tx_info)?;
                            swarm
                                .behaviour_mut()
                                .floodsub
                                .publish(TOPIC.clone(), tx_info_json.as_bytes());
                        }
                        ["ls", "p"] => handle_list_peers(&mut swarm).await,
                        _ => error!("Unknown command"),
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_list_peers(swarm: &mut Swarm<TransactionBehaviour>) {
    info!("Discovered Peers:");
    let nodes = swarm.behaviour().mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }
    unique_peers.iter().for_each(|p| info!("{}", p));
}
