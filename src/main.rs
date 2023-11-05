use std::{sync::{Arc, Mutex}, time::Duration};

use tokio::time::sleep;
mod node;


#[tokio::main]
async fn main() {
    let node = Arc::new(Mutex::new(node::counter::CounterNode::new()));
    let gossip_node_copy = node.clone();

    // Open a task to gossip every 100 ms
    tokio::spawn(async move {
        loop {
            node::counter::gossip(&gossip_node_copy).await;
            sleep(Duration::from_millis(100)).await;
        }
    });

    // Handle requests
    loop {
        let message = node::counter::request_input().await;
        let process_node_copy = node.clone();

        tokio::spawn(async move {
            node::counter::process_message(message, process_node_copy).await;
        });
    }
}
