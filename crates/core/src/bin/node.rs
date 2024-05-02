use eyre::Result;
use futures::stream::StreamExt;
use libp2p::{
    gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux, PeerId, Swarm,
};
use once_cell::sync::Lazy;
use rustic_chain_of_blocks::{
    account::accounts_init,
    blockchain::{get_last_block, Blockchain},
    mempool::{get_all_transaction_reqs, mempool_init},
    transaction::Transaction,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};
use tokio::{io, io::AsyncBufReadExt, select, time::interval};
use tracing_subscriber::EnvFilter;

static TOPIC: Lazy<gossipsub::IdentTopic> = Lazy::new(|| gossipsub::IdentTopic::new("P2P"));

#[derive(NetworkBehaviour)]
struct P2PBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(message_id_fn)
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            Ok(P2PBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm.behaviour_mut().gossipsub.subscribe(&TOPIC)?;

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("💻 Node is live! 💻");

    let _ = accounts_init()?;
    let _ = mempool_init()?;
    let mut blockchain = Blockchain::init()?;
    let mut block_time = interval(Duration::from_secs(12));
    block_time.tick().await;

    loop {
        select! {
            _ = block_time.tick() => {
                let reqs = get_all_transaction_reqs()?;
                let mut txs = vec![];
                if !reqs.is_empty() {
                    for req in reqs {
                        let tx = Transaction::new(req.from, req.to, req.value, req.pk).await?;
                        txs.push(tx);
                    }
                }
                let parent_block = get_last_block()?;
                let _ = blockchain.mine(txs.clone(), &parent_block)?;
            }
            Ok(Some(line)) = stdin.next_line() => {
                handle_input(&mut swarm, line.to_string()).await?;
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("Discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("Peer {peer_id} has expired");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(P2PBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: _id,
                    message,
                })) => {
                    handle_message(&mut swarm, peer_id, message.data).await?
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Node is listening on {address}");
                },
                _ => ()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum P2PMessageType {
    Request,
    Response,
}

#[derive(Debug, Serialize, Deserialize)]
struct P2PMessage {
    id: u64,
    code: Option<u64>,
    want: Option<u64>,
    data: Option<Vec<u8>>,
    random: u64,
    msgtype: P2PMessageType,
}

async fn handle_input(swarm: &mut Swarm<P2PBehaviour>, line: String) -> Result<()> {
    let input: Vec<&str> = line.trim().split_whitespace().collect();

    let msg = P2PMessage {
        id: input[0].parse::<u64>()?,
        code: None,
        want: None,
        data: None,
        random: rand::random::<u64>(),
        msgtype: P2PMessageType::Request,
    };

    let msg_json = serde_json::to_string(&msg)?;

    swarm
        .behaviour_mut()
        .gossipsub
        .publish(TOPIC.clone(), msg_json.as_bytes())?;

    match msg.id {
        0 => println!("Sent Hello message"),
        1 => println!("Sent NewTransaction message"),
        2 => println!("Sent NewBlock message"),
        3 => println!("Sent GetBlock message"),
        4 => println!("Sent Block message"),
        _ => println!("Unknown message type"),
    }
    Ok(())
}

async fn handle_message(
    swarm: &mut Swarm<P2PBehaviour>,
    peer_id: PeerId,
    message: Vec<u8>,
) -> Result<()> {
    let msg: P2PMessage = serde_json::from_slice(&message)?;
    if msg.msgtype == P2PMessageType::Request {
        println!("Received message with id {:?} from peer {peer_id}", msg.id);
        match msg.id {
            // Sending Hello as response for every request as of now.
            _ => {
                let msg_resp = P2PMessage {
                    id: msg.id,
                    code: None,
                    want: None,
                    data: Some("Hello".to_string().as_bytes().to_vec()),
                    random: rand::random::<u64>(),
                    msgtype: P2PMessageType::Response,
                };

                let msg_resp_json = serde_json::to_string(&msg_resp)?;
                swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(TOPIC.clone(), msg_resp_json.as_bytes())?;
                println!("Sent Hello as a response");
            }
        }
    } else {
        // P2PMessageType::Response
        let data = String::from_utf8(msg.data.unwrap())?;
        match msg.id {
            0 => println!("Received {:?} from {peer_id}", data),
            1 => println!("Received {:?} from {peer_id}", data),
            2 => println!("Received {:?} from {peer_id}", data),
            3 => println!("Received {:?} from {peer_id}", data),
            4 => println!("Received {:?} from {peer_id}", data),
            _ => println!("Received unknown message type"),
        }
    }
    Ok(())
}
