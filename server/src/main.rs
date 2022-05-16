use rusqlite::{params, Connection, Result, MappedRows, NO_PARAMS};
use std::{net::TcpListener, thread::spawn};
use std::env; 
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};
mod message_database;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let mut message_database = message_database::init_message_database(
        String::from("main.sql"),
        String::from("test_conversation"), 
    );
    
    let msg = message_database::Message{
        uuid: 0,
        content: String::from("Hello world"), 
        content_type: String::from("text"),
        sender_username: String::from("wredenba"), 
        unix_timestamp: 20
    };

    message_database.save_message(msg, String::from("test_conversation"));

    let new_msg = message_database.get_message_uuid(0, String::from("test_conversation"));

    println!("{}", new_msg.content);
    
    //init_message_database(); 
    /*
    let conn = Connection::open_in_memory().unwrap();

    conn.execute(
        "create table if not exists cat_colors (
             id integer primary key,
             name text not null unique
         )",
        NO_PARAMS,
    ).unwrap();
    conn.execute(
        "create table if not exists cats (
             id integer primary key,
             name text not null,
             color_id integer not null references cat_colors(id)
         )",
        NO_PARAMS,
    ).unwrap();
    */

    /*
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
           
            let msg = websocket.read_message().unwrap();
            println!("Message: {}", msg);
            if msg.is_binary() || msg.is_text() {
                websocket.write_message(msg).unwrap();
            }
            
        });
    }
    */
}
