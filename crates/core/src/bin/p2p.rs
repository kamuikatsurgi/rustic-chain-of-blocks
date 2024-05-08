use alloy_rlp::{Decodable, Encodable};
use eyre::Result;
use futures::stream::StreamExt;
use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, Swarm,
};
use once_cell::sync::Lazy;
use rustic_chain_of_blocks::{
    block::Block,
    p2p::{NBlocks, P2PMessage, VoteOnBlock},
    transaction::Transaction,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};
use tokio::{io, io::AsyncBufReadExt, select};
use tracing_subscriber::EnvFilter;

static TOPIC: Lazy<gossipsub::IdentTopic> =
    Lazy::new(|| gossipsub::IdentTopic::new("Rustic Chain of Blocks"));

#[derive(NetworkBehaviour)]
struct P2PBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
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
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("💻 P2P Node is live! 💻");

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        select! {
            Ok(Some(input)) = stdin.next_line() => {
                handle_input(&mut swarm, input.to_string()).await?;
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("Discovered a new P2P peer {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("P2P Peer {peer_id} has expired");
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
                    println!("P2P Node is live on {address}");
                },
                _ => ()
            }
        }
    }
}

async fn handle_input(swarm: &mut Swarm<P2PBehaviour>, line: String) -> Result<()> {
    let input: Vec<&str> = line.split_whitespace().collect();
    let id = input[0].parse::<u64>()?;
    let code = None;
    let want = None;
    let random = rand::random::<u64>();
    let mut out = Vec::<u8>::new();

    match id {
        0 => {
            "Ping".encode(&mut out);
            let data = Some(out);
            let msg = P2PMessage { id, code, want, data, random };
            let msgjson = serde_json::to_string(&msg)?;
            swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;
            println!("Sent Ping message");
        }
        1 => (),
        2 => {
            let local_address = swarm.local_peer_id().to_base58();
            local_address.encode(&mut out);
            let data = Some(out);
            let msg = P2PMessage { id, code, want, data, random };
            let msgjson = serde_json::to_string(&msg)?;
            swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;
            println!("Sent Address message");
        }
        3 => (),
        4 => (),
        5 => (),
        6 => {
            let want = input[1].parse::<u64>()?;
            let msg = P2PMessage { id, code, want: Some(want), data: None, random };
            let msgjson = serde_json::to_string(&msg)?;
            swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;
            println!("Sent GetBlock message");
        }
        7 => (),
        8 => {
            let msg = P2PMessage { id, code, want, data: None, random };
            let msgjson = serde_json::to_string(&msg)?;
            swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;
            println!("Sent GetLatestBlock message");
        }
        9 => (),
        10 => (),
        _ => println!("Unknown message type"),
    }

    Ok(())
}

async fn handle_message(
    swarm: &mut Swarm<P2PBehaviour>,
    peer_id: PeerId,
    message: Vec<u8>,
) -> Result<()> {
    let recv_msg = serde_json::from_slice::<P2PMessage>(&message)?;
    let code = None;
    let want = None;
    let random = rand::random::<u64>();

    let mut out = Vec::<u8>::new();

    match recv_msg.id {
        0 => (),
        1 => {
            let recv_data = recv_msg.data.unwrap();
            println!(
                "Received {:?} for Ping from {peer_id}",
                String::decode(&mut recv_data.as_slice())?
            );
        }
        2 => (),
        3 => {
            let recv_data = recv_msg.data.unwrap();
            println!(
                "Received {:?} for Address message",
                String::decode(&mut recv_data.as_slice())?
            );
        }
        4 => {
            let recv_tx = recv_msg.data.unwrap();
            let decoded_tx = Transaction::decode(&mut recv_tx.as_slice())?;
            println!("Received a NewTransaction message from {peer_id}\n{:#?}", decoded_tx);
        }
        5 => {
            let recv_block = recv_msg.data.unwrap();
            let decoded_block = Block::decode(&mut recv_block.as_slice())?;
            println!("Received a NewBlock message from {peer_id}\n{:#?}", decoded_block.clone());
            let vote =
                VoteOnBlock { block_number: decoded_block.header.number, vote: "YES".to_string() };
            vote.encode(&mut out);
            let data = Some(out);
            let msg = P2PMessage { id: 10, code, want, data, random };
            let msgjson = serde_json::to_string(&msg)?;
            swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;
            println!("Voting YES for the proposed block");
        }
        6 => (),
        7 => {
            let recv_blocks = recv_msg.data.unwrap();
            let decoded_blocks = NBlocks::decode(&mut recv_blocks.as_slice())?;
            println!("Received a Block message from {peer_id}\n{:#?}", decoded_blocks);
        }
        8 => (),
        9 => {
            let recv_block_number = recv_msg.data.unwrap();
            let decoded_block_number = u64::decode(&mut recv_block_number.as_slice())?;
            println!(
                "Received a GetLatestBlockResponse message from {peer_id} with block number {}",
                decoded_block_number
            );
        }
        10 => (),
        _ => (),
    }

    Ok(())
}
