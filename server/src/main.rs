use rusqlite::{params, Connection, Result, MappedRows, NO_PARAMS};
use std::{net::TcpListener, thread::spawn, thread};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};
use serde_json::{Result as JsonResult, Value};
use std::sync::mpsc::channel;
mod message_database;


// Still working on handing parsing of the message
fn process_message(msg: String) -> Result<message_database::Message, bool>{
    let json_parse: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&msg);
    match json_parse{
        Ok(value)=>{
            let mut msg_struct =  message_database::Message {
                uuid: 0, 
                content: value["content"].to_string(), 
                content_type: value["content_type"].to_string(), 
                sender_username: value["sender_username"].to_string(), 
                unix_timestamp: (value["unix_timestamp"]["secs_since_epoch"]
                                    .to_string().parse::<u32>().unwrap())
            };
            return Ok(msg_struct)
        }
        Err(e)=>{
            println!("Message could not be parsed:  {}", e);
            return Err(false);
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

fn insert_message_database(msg: String, tx_clone: std::sync::mpsc::Sender<message_database::Message>) -> bool{
    let message_process = process_message(msg);
    match message_process {
        Ok(data)=>{
            tx_clone.send(data); 
            return true; 
        }
        Err(e)=> {
            return false; 
        }
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
                        insert_message_database(msg.to_string(), tx_clone.clone()); 
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
