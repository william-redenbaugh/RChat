use rusqlite::{params, Connection, Result, MappedRows, NO_PARAMS};
use std::{net::TcpListener, thread::spawn, thread};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};
use serde_json::{Result as JsonResult, Value};
use std::sync::mpsc::channel;
mod message_database;
use std::time::{Duration, SystemTime};

fn abs_int(x: i64) -> u64 {
    if x < 0{
        return (x * -1) as u64; 
    }

    return x as u64; 
}

// Still working on handing parsing of the message
fn input_message_database(tx_clone: std::sync::mpsc::Sender<message_database::Message>, value: serde_json::Value) -> bool{
    let mut msg_struct =  message_database::Message {
        uuid: 0, 
        content: value["content"].to_string(), 
        content_type: value["content_type"].to_string(), 
        sender_username: value["sender_username"].to_string(), 
        unix_timestamp: (value["unix_timestamp"]["secs_since_epoch"]
                            .to_string().parse::<u32>().unwrap())
    };
    let now = SystemTime::now();
    if abs_int((msg_struct.unix_timestamp as i64 - now.elapsed().unwrap().as_secs() as i64)) >  130{
        return false;
    }
    
    match tx_clone.send(msg_struct) {
        Ok(a)=> return true, 
        Err(e)=>{
            println!("Error sending data to database thread... {}", e);
            return false; 
        }
    }
}

fn database_handler_thread(rx_msg:  std::sync::mpsc::Receiver<message_database::Message>){
    let mut message_database = message_database::init_message_database(true, 
        String::from("msg.sql"), 
        String::from("wredenba")); 

    loop{
        let msg_req = rx_msg.recv(); 
        match msg_req{
            Ok(msg)=> {
                //message_database.save_message(msg, String::from("wredenba"));
                println!("{}", msg.content);
            }, 
            Err(e)=>{
                println!("Had issues getting message from pipeline: {}", e);
            }
        }
    }
}

fn process_incoming_packet(msg: String, tx_clone: std::sync::mpsc::Sender<message_database::Message>) -> bool{

    let json_req: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&msg);
    let mut json_parse: serde_json::Value;

    match json_req {
        Ok(data)=>json_parse = data, 
        Err(e)=>{
            println!("Error parsing JSON: {}", e); 
            return false; 
        }
    }
    
    if json_parse["request_type"].to_string() == "send_msg"{
        return input_message_database(tx_clone, json_parse.clone());        
    }
    else{
        return false; 
    }

}

fn chatserver_handler_thread(tx_mesg: std::sync::mpsc::Sender<message_database::Message>){
    let server = TcpListener::bind("127.0.0.1:1212").unwrap();
    for stream in server.incoming() {
        let tx_clone = tx_mesg.clone(); 
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
                            process_incoming_packet(msg.to_string(), tx_clone.clone()); 
                        }
                    }
                }
            }
        });
    }
}

fn main() {

    // Create a simple streaming channel
    let (tx_mesg, rx_msg) = channel();
    spawn(|| {database_handler_thread(rx_msg)});
    chatserver_handler_thread(tx_mesg); 
}
