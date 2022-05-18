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
    socket: WebSocket<MaybeTlsStream<TcpStream>>
}

fn new_connection(ip_in: String, port_in: String)->MessengerConnection{
    
    let mut str = String::from("ws://"); 
    str.push_str(&ip_in);
    str.push(':');
    str.push_str(&port_in.as_str()); 
    str.push_str(&String::from("/socket"));

    let (mut m_socket, response) = connect(url::Url::parse(&str).unwrap()).expect("Cannot connect to port... ");

    return MessengerConnection { 
        ip: ip_in,
        port: port_in ,
        socket: m_socket
    };
}

impl MessengerConnection{
    pub fn get_messages(&mut self) -> String{

        return String::from("");
    }

    pub fn send_message(&mut self, msg: String) -> bool{
        self.socket.write_message(Message::Text(msg.clone().into())).unwrap();
        let msg_ack = self.socket.read_message().expect("Error reading message").to_string();
        
        if !msg.eq(&msg_ack) {
            println!("Message wasn't sent successfully...");
            return false; 
        }

        return true; 
    }

    pub fn close_connection(&mut self){
        self.socket.close(None);
    }
}


fn main() {
    let mut conn = new_connection(String::from("localhost"), String::from("1212"));
    let mut user_message = user_message_handler(String::from("wredenba"));

    let mut input = String::new(); 
    println!("What is your message?: ");
    let input_type = std::io::stdin().read_line(&mut input).unwrap();
    
    conn.send_message(user_message.message(input));

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

