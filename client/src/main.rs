use tungstenite::{accept, connect, stream::MaybeTlsStream, Error, Message, WebSocket};
use std::net::TcpStream;
use url::Url; 
use serde_json::json;
use std::time::{Duration, SystemTime};
use ctrlc; 

fn user_message_handler(username_r: String) -> UserMessage{
    return UserMessage{
        username: username_r
    }; 
}

pub struct UserMessage{
    username: String
}

impl UserMessage{
    fn message(&mut self, mut msg: String) ->String{
        msg.pop();
        let msg_json = json!({
            "request_type": "send_msg",
            "content": msg, 
            "content_type": "text", 
            "sender_username": &self.username, 
            "unix_timestamp": SystemTime::now()
        });
        return msg_json.to_string();   
    }
}

pub struct MessengerConnection{
    ip: String, 
    port: String, 
    socket: WebSocket<MaybeTlsStream<TcpStream>>, 
    username: String
}

fn new_connection(m_username: String, ip_in: String, port_in: String)->MessengerConnection{
    
    let mut str = String::from("ws://"); 
    str.push_str(&ip_in);
    str.push(':');
    str.push_str(&port_in.as_str()); 
    str.push_str(&String::from("/socket"));

    let (mut m_socket, response) = connect(url::Url::parse(&str).unwrap()).expect("Cannot connect to port... ");

    return MessengerConnection { 
        ip: ip_in,
        port: port_in ,
        socket: m_socket, 
        username: m_username
    };
}

impl MessengerConnection{
    pub fn get_messages(&mut self) -> String{
        let msg_json = json!({
            "request_type": "get_msg_list"
        });
        let msg = msg_json.to_string();
        self.socket.write_message(Message::Text(msg.clone().into())).unwrap();

        return self.socket.read_message().unwrap().to_string(); 
    }

    pub fn send_message(&mut self, msg: String) -> bool{
        println!("msg: {}", msg);
        self.socket.write_message(Message::Text(msg.clone().into())).unwrap();
        
        return true; 
    }

    pub fn close_connection(&mut self){
        self.socket.close(None);
    }
}

use std::{thread, time};


fn main() {
    
    let mut input = String::new(); 
    println!("What is your message?: ");
    let input_type = std::io::stdin().read_line(&mut input).unwrap();
    let mut conn = new_connection(String::from("wredenba"), String::from("localhost"), String::from("1212"));
    let mut user_message = user_message_handler(String::from("wredenba"));
    conn.send_message(user_message.message(input));

    let ten_millis = time::Duration::from_millis(100);
    thread::sleep(ten_millis);

    let messages = conn.get_messages(); 
    println!("{}", messages);

    conn.close_connection();
}

// Returns true if we are writing, false if we are reading
fn handle_input() -> bool{
    let mut input_l = String::new(); 
    println!("Please Enter: \"R\": for reading all messages, \"W\": for writing a new message");
    let input_type = std::io::stdin().read_line(&mut input_l).unwrap();

    if (input_l == "R") | (input_l == "r"){
        return false; 
    }
    else{
        return true;    
    }
}