use std::{net::IpAddr, time::Duration};

use crate::types::VoiceNode;

use futures::{FutureExt, future::join_all};
use reqwest::Url;
use tokio::net::lookup_host;

pub async fn ping_node(node: &VoiceNode) -> Option<Duration> {
    let address = match node.public_url.parse::<IpAddr>() {
        Ok(addr) => addr,
        Err(_) => {
            let url = node.public_url.parse::<Url>().ok()?;

            let mut host = url.host_str()?.to_string();
            host.push_str(":0");

            lookup_host(&host).await.ok()?.next()?.ip()
        }
    };

    let result = tokio::task::spawn_blocking(move || {
        ping::new(address)
            .socket_type(ping::SocketType::DGRAM)
            .ttl(128)
            .timeout(Duration::from_secs(1))
            .send()
    })
    .await
    .ok()?
    .ok()?;

    Some(result.rtt)
}

pub async fn find_closest_node(nodes: &[VoiceNode]) -> Option<&VoiceNode> {
    let futs = nodes.iter().map(|node| ping_node(node).boxed_local());

    let mut o = join_all(futs)
        .await
        .into_iter()
        .zip(nodes)
        .filter_map(|(rtt, node)| rtt.map(|rtt| (rtt, node)))
        .collect::<Vec<_>>();

    o.sort_by(|(rtt1, _), (rtt2, _)| rtt1.cmp(rtt2));

    o.into_iter().next().map(|(_, node)| node)
}
