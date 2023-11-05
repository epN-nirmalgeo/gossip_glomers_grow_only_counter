use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde_json::json;
use tokio::signal;
use tokio::time::{sleep, Duration};
use super::message::{Message, MessageType, MessageBody};

pub struct CounterNode {
    pub id: String,
    pub counter: Arc<Mutex<i64>>,
    pub nodes: Vec<String>,
    pub other_node_counter: Arc<Mutex<HashMap<String, i64>>>,
}

impl CounterNode {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            counter: Arc::new(Mutex::new(0)),
            nodes: Vec::new(),
            other_node_counter: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub async fn request_input() -> Message {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    let request: Message = serde_json::from_str(&line).unwrap();
    request
}

fn generate_response(request: &Message, response_type: MessageType) -> Message {
    let mut params = HashMap::new();
    let mut response = Message {
        src: request.dest.to_owned(),
        dest: request.src.to_owned(),
        body: MessageBody { typ: response_type, message_params: HashMap::new()}
    };

    params.insert("in_reply_to".to_owned(), request.body.message_params.get("msg_id").unwrap().clone().to_owned());
    // for now insert a dummy entry as msg_id, maybe snowflake algorithm here?
    params.insert("msg_id".to_owned(), json!(1));
    response.body.message_params = params;
    
    response
}

async fn gossip(node: &Arc<Mutex<CounterNode>>) {
    let node = node.lock().unwrap();
    let counter = node.counter.lock().unwrap();
    for dest in &node.nodes {
        let mut gossip = Message {
            src: node.id.to_owned(),
            dest: dest.to_owned(),
            body: MessageBody { 
                typ: MessageType::Gossip, 
                message_params: HashMap::new(), 
            }
        };
        gossip.body.message_params.insert("value".to_owned(), json!(*counter));

        let gossip = serde_json::to_string(&gossip).unwrap();
        println!("{}", gossip);
    }

    let _ = sleep(Duration::from_millis(1000));
}

pub async fn gossip_handler(node: Arc<Mutex<CounterNode>>) {

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                eprintln!("break gossip");
                return;
            }
            _ = gossip(&node) => {
                //eprintln!("proceed with gossip");
            }

        }
    }

}

pub async fn process_message(request: Message, node: Arc<Mutex<CounterNode>>) {

    match &request.body.typ {
        MessageType::Add => {
            if let Some(value) = request.body.message_params.get("delta") {
                let node = node.lock().unwrap();
                let mut counter = node.counter.lock().unwrap();
                *counter += value.as_i64().unwrap();
            }

            let response = generate_response(&request, MessageType::AddOk);
            let response = serde_json::to_string(&response).unwrap();
            println!("{}", response);
        }

        MessageType::Init => {
            let mut node = node.lock().unwrap();
            node.id = request.dest.to_owned();
            let response = generate_response(&request, MessageType::InitOk);
            let response = serde_json::to_string(&response).unwrap();
            println!("{}", response);

            node.id = request.body.message_params.get("node_id").unwrap().as_str().unwrap().to_owned();
            
            let received_nodes = request.body.message_params.get("node_ids").unwrap().as_array().unwrap();
            for received_node in received_nodes {
                node.nodes.push(received_node.as_str().unwrap().to_owned());
            }
        }

        MessageType::Read => {
            let mut response = generate_response(&request, MessageType::ReadOk);
            let node = node.lock().unwrap();
            let counter = node.counter.lock().unwrap();
            let other_node_counters = node.other_node_counter.lock().unwrap();
            let mut sum_of_nodes = *counter;
            for (key, value) in &*other_node_counters {
                if key == &node.id {
                    continue;
                }
                sum_of_nodes += value;
            }

            response.body.message_params.insert("value".to_owned(), json!(sum_of_nodes));
            let response = serde_json::to_string(&response).unwrap();
            println!("{}", response);
        }

        MessageType::Gossip => {
            let node = node.lock().unwrap();
            let src = request.src.to_owned();
            let value = request.body.message_params.get("value").unwrap().as_i64().unwrap();
            let mut other_node_counter = node.other_node_counter.lock().unwrap();
            other_node_counter.insert(src, value);
        }

        _ => {
            todo!("test");
        }
    }
}
