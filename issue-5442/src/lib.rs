use std::time::Duration;

use futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{ping, webtransport_websys, SwarmBuilder};
use multiaddr::Multiaddr;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
use web_sys::{window, Response};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn substream_dial_error() {
    let mut swarm = SwarmBuilder::with_new_identity()
        .with_wasm_bindgen()
        .with_other_transport(|local_keypair| {
            let config = webtransport_websys::Config::new(local_keypair);
            webtransport_websys::Transport::new(config)
        })
        .unwrap()
        .with_behaviour(|_| {
            let config = ping::Config::new()
                .with_timeout(Duration::from_secs(1))
                .with_interval(Duration::from_secs(1));
            ping::Behaviour::new(config)
        })
        .unwrap()
        .with_swarm_config(|config| config.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    let addr = fetch_server_addr().await;
    swarm.dial(addr).unwrap();

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(ping::Event {
                result: Err(ping::Failure::Timeout),
                ..
            }) => {
                panic!("Timeout shouldn't happen");
            }
            SwarmEvent::Behaviour(ping::Event { result: Err(_), .. }) => {
                break;
            }
            SwarmEvent::Behaviour(ping::Event { result: Ok(_), .. }) => {
                panic!("Successful ping shouldn't happen");
            }
            _ => {}
        }
    }
}

/// Helper that returns the multiaddress of the other peer
///
/// It fetches the multiaddress via HTTP request to
/// 127.0.0.1:4455.
async fn fetch_server_addr() -> Multiaddr {
    let url = "http://127.0.0.1:4455/";
    let window = window().expect("failed to get browser window");

    let value = JsFuture::from(window.fetch_with_str(url))
        .await
        .expect("fetch failed");
    let resp = value.dyn_into::<Response>().expect("cast failed");

    let text = resp.text().expect("text failed");
    let text = JsFuture::from(text).await.expect("text promise failed");

    text.as_string()
        .filter(|s| !s.is_empty())
        .expect("response not a text")
        .parse()
        .unwrap()
}
