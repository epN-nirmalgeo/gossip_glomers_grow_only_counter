use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde_json::json;

use super::message::{Message, MessageType, MessageBody};

type Node = Arc<Mutex<CounterNode>>;

pub struct CounterNode {
    id: String,
    counter: i64,
}

impl CounterNode {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            counter: 0,
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

pub async fn process_message(request: Message, node: Node) {

    match &request.body.typ {
        MessageType::Add => {
            if let Some(value) = request.body.message_params.get("delta") {
                let mut node = node.lock().unwrap();

                let temp_value = node.counter + value.as_i64().unwrap();

                let mut replicate_request = generate_response(&request, MessageType::Replicate);
                replicate_request.body.message_params.insert("value".to_owned(), json!(temp_value));
                // replicate to all nodes and only then update current counter
                for dest in vec!["n0", "n1", "n2", "n3"] {
                    if node.id != dest {
                        replicate_request.src = node.id.clone();
                        replicate_request.dest = dest.to_string();
                        eprintln!("{:?}", replicate_request);
                        let replicate_request = serde_json::to_string(&replicate_request).unwrap();
                        println!("{}", replicate_request);
                    }
                }
        
                node.counter = temp_value;
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

            // request replicate from a random node other than itself
            for dest in vec!["n0", "n1", "n2", "n3"] {
                if node.id != dest {
                    let mut request = generate_response(&request, MessageType::RequestReplication);
                    request.src = node.id.to_owned();
                    request.dest = dest.to_string();
                    let request = serde_json::to_string(&request).unwrap();
                    eprintln!("{}", request);
                    println!("{}", request);
                    break;
                }
            }
        }

        MessageType::RequestReplication => {
            let node = node.lock().unwrap();
            let mut response = generate_response(&request, MessageType::Replicate);
            response.src = request.dest;
            response.dest = request.src;
            response.body.message_params.insert("value".to_owned(), json!(node.counter));
            let response = serde_json::to_string(&response).unwrap();
            println!("{}", response);
        }

        MessageType::Read => {
            let mut response = generate_response(&request, MessageType::ReadOk);
            let node = node.lock().unwrap();
            response.body.message_params.insert("value".to_owned(), json!(node.counter));
            let response = serde_json::to_string(&response).unwrap();
            println!("{}", response);
        }

        MessageType::Replicate => {
            let mut node = node.lock().unwrap();    
            let value = request.body.message_params.get("value").unwrap().as_i64().unwrap();
            node.counter = value;
        }

        _ => {
            todo!("test");
        }
    }
}
