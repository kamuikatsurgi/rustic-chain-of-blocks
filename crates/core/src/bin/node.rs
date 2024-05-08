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
    account::accounts_init,
    block::Block,
    blockchain::{get_last_block, get_last_n_blocks, Blockchain},
    mempool::{get_all_transaction_reqs, mempool_init},
    p2p::{P2PMessage, VoteOnBlock},
    transaction::Transaction,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};
use tokio::{io, select, time::interval};
use tracing_subscriber::EnvFilter;

static TOPIC: Lazy<gossipsub::IdentTopic> =
    Lazy::new(|| gossipsub::IdentTopic::new("Rustic Chain of Blocks"));

#[derive(NetworkBehaviour)]
struct RCOBBehaviour {
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
            Ok(RCOBBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm.behaviour_mut().gossipsub.subscribe(&TOPIC)?;

    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("🦀 Blockchain is live! 🦀");

    accounts_init()?;
    mempool_init()?;
    let mut blockchain = Blockchain::init()?;

    let mut block_time = interval(Duration::from_secs(5));
    block_time.tick().await;

    let mut proposed_block = Block::genesis()?;
    let mut yes_votes: u64 = 0;
    let mut proposed = false;

    loop {
        select! {
            _ = block_time.tick() => {
                if !proposed {
                    let reqs = get_all_transaction_reqs()?;
                    let mut txs = vec![];
                    if !reqs.is_empty() {
                        for req in reqs {
                            let tx = Transaction::new(req.from, req.to, req.value, req.pk).await?;
                            txs.push(tx.clone());
                            handle_send_tx(&mut swarm, tx.clone()).await?;
                        }
                    }
                    let parent_block = get_last_block()?;
                    proposed_block = blockchain.propose_block(txs.clone(), &parent_block)?;
                    handle_send_block(&mut swarm, 5, proposed_block.clone()).await?;
                    proposed = true;
                } else {
                    if yes_votes > (swarm.connected_peers().count() / 2).try_into()? {
                        println!("Got majority votes, finalizing the block...");
                        blockchain.commit_block(proposed_block.clone())?;
                    }
                    yes_votes = 0;
                    proposed = false;
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(RCOBBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("Discovered a new P2P peer {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(RCOBBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("P2P Peer {peer_id} has expired");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(RCOBBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: _id,
                    message,
                })) => {
                    handle_message(&mut swarm, peer_id, message.data, &mut yes_votes).await?
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Blockchain is live on {address}");
                },
                _ => ()
            }
        }
    }
}

async fn handle_message(
    swarm: &mut Swarm<RCOBBehaviour>,
    peer_id: PeerId,
    message: Vec<u8>,
    yes_votes: &mut u64,
) -> Result<()> {
    let recv_msg = serde_json::from_slice::<P2PMessage>(&message)?;
    let code = None;
    let want = None;
    let random = rand::random::<u64>();

    let mut out = Vec::<u8>::new();

    match recv_msg.id {
        0 => {
            "Pong".encode(&mut out);
            let data = Some(out);
            let msg = P2PMessage { id: 1, code, want, data, random };
            let msgjson = serde_json::to_string(&msg)?;
            swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;
            println!("Sent Pong in response to Ping from {peer_id}");
        }
        1 => (),
        2 => {
            let address = swarm.local_peer_id().to_base58();
            address.encode(&mut out);
            let data = Some(out);
            let msg = P2PMessage { id: 3, code, want, data, random };
            let msgjson = serde_json::to_string(&msg)?;
            swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;
            println!("Sent {} in response to Address from {peer_id}", address);
        }
        3 => (),
        4 => (),
        5 => (),
        6 => {
            let num_blocks = recv_msg.want.unwrap();
            let blocks = get_last_n_blocks(num_blocks.try_into()?)?;
            blocks.encode(&mut out);
            let data = Some(out);
            let msg = P2PMessage { id: 7, code, want, data, random };
            let msgjson = serde_json::to_string(&msg)?;
            swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;
            println!("Sent Block in response to GetBlock from {peer_id}");
        }
        7 => (),
        8 => {
            let block_num = get_last_block()?.header.number;
            block_num.encode(&mut out);
            let data = Some(out);
            let msg = P2PMessage { id: 9, code, want, data, random };
            let msgjson = serde_json::to_string(&msg)?;
            swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;
            println!("Sent GetLatestBlockResponse in response to GetLatestBlock from {peer_id}");
        }
        9 => (),
        10 => {
            let recv_data = recv_msg.data.unwrap();
            let recv_vote = VoteOnBlock::decode(&mut recv_data.as_slice())?;
            println!("Received {} for block number {}", recv_vote.vote, recv_vote.block_number);
            if recv_vote.vote == "YES" {
                *yes_votes += 1;
            }
        }
        _ => println!("Unknown message type"),
    }

    Ok(())
}

async fn handle_send_block(swarm: &mut Swarm<RCOBBehaviour>, id: u64, block: Block) -> Result<()> {
    let mut out = Vec::<u8>::new();
    block.encode(&mut out);
    let msg =
        P2PMessage { id, code: None, want: None, data: Some(out), random: rand::random::<u64>() };
    let msgjson = serde_json::to_string(&msg)?;
    swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;

    Ok(())
}

async fn handle_send_tx(swarm: &mut Swarm<RCOBBehaviour>, tx: Transaction) -> Result<()> {
    let mut out = Vec::<u8>::new();
    tx.encode(&mut out);
    let msg = P2PMessage {
        id: 4,
        code: None,
        want: None,
        data: Some(out),
        random: rand::random::<u64>(),
    };
    let msgjson = serde_json::to_string(&msg)?;
    swarm.behaviour_mut().gossipsub.publish(TOPIC.clone(), msgjson.as_bytes())?;

    Ok(())
}
