use rusqlite::{params, Connection, Result, NO_PARAMS};
use std::{net::TcpListener, thread::spawn};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};

struct Message{
    uuid: i64, 
    content: String, 
    content_type: String,
    sender_username: String, 
    unix_timestamp: u32
}

struct MessageDatabase{
    conn: Connection
}

fn init_message_database() -> MessageDatabase{
    let mut conn_r = Connection::open_in_memory().unwrap();

        conn_r.execute(
            "CREATE TABLE if not exists message_list (
                    uuid              INTEGER PRIMARY KEY,
                    content           TEXT NOT NULL,
                    content_type      TEXT NOT NULL,
                    sender_username   TEXT NOT NULL,
                    unix_timestamp    INTEGER
                    )",
            NO_PARAMS,
        ).unwrap(); 


    return MessageDatabase{
        conn: conn_r
    };
}

impl MessageDatabase{
    pub fn save_message(&mut self, message: Message) -> bool{
        self.conn.execute("INSERT INTO message_list (uuid, content, content_type, sender_username, unix_timestamp) (?1, ?2, ?3, ?4, ?5)", 
        params![message.uuid, 
        &message.content, 
        &message.content_type, 
        &message.sender_username,
        message.unix_timestamp]).unwrap(); 

        return true; 
    }

    pub fn get_message(&mut self, uuid: i64) -> Message{
        let mut message =  Message{
            uuid: -1, 
            content: String::from(""),
            content_type: String::from(""), 
            sender_username: String::from(""),
            unix_timestamp: 0
        }; 

        return message; 
    }
}

fn main() {
    let mut message_database = init_message_database();
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
