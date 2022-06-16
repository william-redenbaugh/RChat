use rusqlite::{Result};
use std::{net::TcpListener, net::TcpStream, thread::spawn};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    protocol::WebSocket
};
use std::sync::mpsc::channel;
mod message_database;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::mpsc::{Sender, Receiver};

pub enum DatabaseServerReqType{
    REQUEST_SEND_MESSAGE,
    REQUEST_ALL_MESSAGES, 
    REQUEST_MESSAGE_TIMESTAMP, 
    REQUEST_MESSAGE_UUID, 
    REQUEST_MESSAGE_USER
}

pub struct DatabaseServerReq{
    pub upper_bounds: i64, 
    pub lower_bounds: i64, 
    pub user_req: String, 
    pub req_type: DatabaseServerReqType, 
    pub return_pipe: Sender<Vec<message_database::Message>>   
}

fn new_database_server_req(req_type_n: DatabaseServerReqType) -> (DatabaseServerReq, Receiver<Vec<message_database::Message>>){
    let (send_pipe, return_pipe) = channel();
    return (DatabaseServerReq{
        upper_bounds: 0,
        lower_bounds: 0, 
        user_req: String::new(), 
        req_type: req_type_n,
        return_pipe: send_pipe
    }, return_pipe);
}

const INT64_BITS: i64 = 64; 

fn rotate_bits(n: i64, d: i8) -> i64{
    return (n << d)|(n >> (INT64_BITS - d as i64));
}

fn abs_int(x: i64) -> u64 {
    if x < 0{
        return (x * -1) as u64; 
    }

    return x as u64; 
}

fn get_message_database(tx_req_clone: Sender<DatabaseServerReq>) -> Vec<message_database::Message>{
    let (database_server_req, return_pipe) = new_database_server_req(DatabaseServerReqType::REQUEST_SEND_MESSAGE); 

    match tx_req_clone.send(database_server_req){
        Ok(_)=>{
            match return_pipe.recv(){
                Ok(msg_list)=>return msg_list, 
                Err(e)=>println!("Error getting message database list back to network handler thread: {}", e)
            }
        }
        Err(e)=>{
            println!("Error sending request data to database thread... {}", e);
        }
    }

    let msg_list_empty = Vec::new(); 
    return msg_list_empty
}

fn input_message_database(tx_clone: Sender<message_database::Message>, tx_req_clone: Sender<DatabaseServerReq>, value: serde_json::Value) -> bool{
    let mut msg_struct =  message_database::Message {
        uuid: 0, 
        content: value["content"].to_string(), 
        content_type: value["content_type"].to_string(), 
        sender_username: value["sender_username"].to_string(), 
        unix_timestamp: (value["unix_timestamp"]["secs_since_epoch"]
                            .to_string().parse::<u32>().unwrap())
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    if abs_int(msg_struct.unix_timestamp as i64 - now.as_secs() as i64) >  130{
        println!("Message process timout!");
        return false;
    }

    let val = rotate_bits(now.as_nanos() as i64, 13); 
    msg_struct.uuid = val;

    let (database_server_req, _) = new_database_server_req(DatabaseServerReqType::REQUEST_SEND_MESSAGE); 

    match tx_req_clone.send(database_server_req){
        Ok(_)=>{
            // We were able to send the request properly
        }
        Err(e)=>{
            println!("Error sending request data to database thread... {}", e);
            return false; 
        }
    }

    match tx_clone.send(msg_struct) {
        Ok(_a)=> return true, 
        Err(e)=>{
            println!("Error sending data to database thread... {}", e);
            return false; 
        }
    }
}

fn database_handler_thread(rx_msg:  Receiver<message_database::Message>, rx_mesg_req: Receiver<DatabaseServerReq>){
    let mut message_database = message_database::init_message_database(true, 
        String::from("msg.sql"), 
        String::from("wredenba")); 

    loop{
        let msg_req_type = rx_mesg_req.recv();
        match msg_req_type{
            Ok(msg_type)=> {
                match  msg_type {
                    REQUEST_SEND_MESSAGE=>{
                        let msg_req = rx_msg.recv(); 
                        match msg_req{
                            Ok(msg)=>{
                                message_database.save_message(msg, String::from("wredenba"));
                            }
                            Err(e)=>{
                                println!("Had issues getting message request from pipeline: {}", e);
                            }
                        }
                    },
                    REQUEST_ALL_MESSAGES=>{
                       let _msg_list = message_database.get_all_messages(String::from("wredenba"));
                    }, 
                    REQUEST_MESSAGE_TIMESTAMP=>{}, 
                    REQUEST_MESSAGE_UUID=>{}, 
                    REQUEST_MESSAGE_USER=>{},
                }
            }, 
            Err(e)=>{
                println!("Had issues getting message request from pipeline: {}", e);
            }
        }
    }
}

fn process_incoming_packet(socket: &mut WebSocket<TcpStream>, msg: String, tx_clone: Sender<message_database::Message>, tx_req_clone: Sender<DatabaseServerReq>) -> bool{
    let json_req: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&msg);
    let json_parse: serde_json::Value;

    match json_req {
        Ok(data)=>json_parse = data, 
        Err(e)=>{
            println!("Error parsing JSON: {}", e); 
            return false; 
        }
    }
    
    match json_parse["request_type"].to_string().as_str() {
        "\"send_msg\"" =>{
            return input_message_database(tx_clone, tx_req_clone, json_parse.clone());
        },
        "\"get_msg_list\"" =>{
            let msg_list = get_message_database(tx_req_clone);
            let msg_list_string = serde_json::to_string(&msg_list).unwrap();
            
            match socket.write_message(tungstenite::Message::Text(msg_list_string)){
                Ok(_)=>return true, 
                Err(e)=>{
                    println!("We weren't able to send message back to client: {}", e);
                    return false; 
                }
            }
        },     
        _ => return false
    }
}

fn chatserver_handler_thread(tx_mesg: Sender<message_database::Message>, tx_mesg_req: Sender<DatabaseServerReq>){
    let server = TcpListener::bind("127.0.0.1:1212").unwrap();
    for stream in server.incoming() {
        let tx_clone = tx_mesg.clone();
        let tx_req_clone = tx_mesg_req.clone(); 
        spawn(move || {
            let callback = |req: &Request, mut response: Response| {
                println!("Received a new ws handshake");
                println!("The request's path is: {}", req.uri().path());
                println!("The request's headers are:");
                for (ref header, _value) in req.headers() {
                    println!("* {}", header);
                }

                // Let's add an additional header to our response to the client.
                let headers = response.headers_mut();
                headers.append("MyCustomHeader", ":)".parse().unwrap());
                headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());
                Ok(response)
            };

            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();
            loop{
                let websocket_req = websocket.read_message(); 
                match websocket_req{
                    Err(e)=>{
                        println!("Socket has been closed: {}", e); 
                        break; 
                    }
                    Ok(msg)=>{
                        if msg.len() > 0 {
                            process_incoming_packet(&mut websocket, msg.to_string(), tx_clone.clone(), tx_req_clone.clone()); 
                        }
                    }
                }
            }
        });
    }
}

fn main() {
    // Create a simple message_streaming_channel
    let (tx_mesg, rx_msg) = channel();

    let (tx_mesg_req, rx_mesg_req) = channel(); 

    spawn(|| {database_handler_thread(rx_msg, rx_mesg_req)});
    chatserver_handler_thread(tx_mesg, tx_mesg_req); 
}
