use eyre::Result;
use futures::stream::StreamExt;
use libp2p::{
    gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux, PeerId,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};
use tokio::{io, io::AsyncBufReadExt, select};
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

    println!("Node is live!");

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                let input = handle_input(line.to_string()).await?;
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(TOPIC.clone(), input.as_bytes()) {
                    println!("Publish error: {e:?}");
                }
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
                    if let Err(e) = handle_message(message.data, peer_id).await {
                        println!("Error handling message: {:?}", e);
                    }
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Node is listening on {address}");
                },
                _ => ()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct P2PRequest {
    id: u64,
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct P2PResponse {
    id: u64,
    data: Vec<u8>,
}

async fn handle_input(line: String) -> Result<String> {
    let input: Vec<&str> = line.trim().split_whitespace().collect();

    let request = P2PRequest {
        id: input[0].to_string().parse::<u64>()?,
        data: input[1].to_string().as_bytes().to_vec(),
    };

    match request.id {
        0 => println!("Sent Hello message"),
        1 => println!("Sent NewTransaction message"),
        2 => println!("Sent NewBlock message"),
        3 => println!("Sent GetBlock message"),
        4 => println!("Sent Block message"),
        _ => println!("Unknown message type"),
    }

    let json_request = serde_json::to_string(&request)?;

    Ok(json_request)
}

async fn handle_message(message: Vec<u8>, peer_id: PeerId) -> Result<()> {
    let response: P2PResponse = serde_json::from_slice(&message)?;

    match response.id {
        0 => println!("Received Hello from {peer_id}"),
        1 => println!("Received NewTransaction from {peer_id}"),
        2 => println!("Received NewBlock from {peer_id}"),
        3 => println!("Received GetBlock from {peer_id}"),
        4 => println!("Received Block from {peer_id}"),
        _ => println!("Unknown message type!"),
    }

    Ok(())
}
