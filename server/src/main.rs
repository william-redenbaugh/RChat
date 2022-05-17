use rusqlite::{params, Connection, Result, MappedRows, NO_PARAMS};
use std::{net::TcpListener, thread::spawn};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};
use serde_json::{Result as JsonResult, Value};
mod message_database;


// Still working on handing parsing of the message
/*
fn handle_message(msg: String, database: &mut message_database::MessageDatabase) -> Result<message_database::Message, bool>{
    let json_parse: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&msg);
    match json_parse{
        Ok(value)=>{
            let mut msg_struct; 
            return Ok()
        }
        Err(e)=>{
            println!("Message could not be sent: {}", e);
        }
    }
}
*/

fn main() {
    let server = TcpListener::bind("127.0.0.1:1212").unwrap();
    for stream in server.incoming() {
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
                let msg = websocket.read_message().unwrap();
                println!("Message: {}", msg);
                if msg.is_binary() || msg.is_text() {
                    websocket.write_message(msg.clone()).unwrap();
                }
                //handle_message(msg.to_string(), &mut database);
            }
        });
    }
}
