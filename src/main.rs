use std::sync::{Arc, Mutex};
mod node;


#[tokio::main]
async fn main() {
    let node = Arc::new(Mutex::new(node::counter::CounterNode::new()));
    loop {
        let message = node::counter::request_input().await;
        let node = node.clone();

        tokio::spawn(async {
            node::counter::process_message(message, node).await;
        });
    }
}
