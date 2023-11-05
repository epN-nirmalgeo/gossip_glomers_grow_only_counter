use std::sync::{Arc, Mutex};
mod node;


#[tokio::main]
async fn main() {
    let node = Arc::new(Mutex::new(node::counter::CounterNode::new()));
    let mut started_gossip = false;

    loop {
        let message = node::counter::request_input().await;
        let process_node_copy = node.clone();
        let gossip_node_copy = node.clone();

        tokio::spawn(async {
            node::counter::process_message(message, process_node_copy).await;
        });

        if !started_gossip {
            tokio::spawn(async {
                node::counter::gossip_handler(gossip_node_copy).await;
            });
            started_gossip = true;
        }
    }
}
