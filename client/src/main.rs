use std::net::SocketAddr;
use tarpc::{client, context, tokio_serde::formats::Json};
pub struct MessengerConnection{
    ip: String, 
    port: String, 
    server_addr: SocketAddr,
}

fn new_connection(ip_in: String, port_in: String)->MessengerConnection{
    
    let mut str = ip_in.clone();
    str.push(':'); 
    str.push_str(&port_in.as_str()); 

    let mut serv_addr = str.parse().expect("Could not parse"); 

    let mut transport = tarpc::serde_transport::tcp::connect(serv_addr, Json::default);

    return MessengerConnection { 
        ip: ip_in,
        port: port_in ,
        server_addr: serv_addr
    };
}

impl MessengerConnection{
    pub fn get_messages(&mut self) -> String{

        return String::from("");
    }

    pub fn send_message(&mut self, msg: String){
        
    }
}

#[tokio::main]
fn main() {

    let mut conn = new_connection(String::from("192.168.1.75"), String::from("1212"));
    let is_writing = handle_input(); 
    
    if is_writing {
        let mut input = String::new(); 
        let input_type = std::io::stdin().read_line(&mut input).unwrap();
        println!("What is your message?: ");
        conn.send_message(input);
    }
    else{

        let messages = conn.get_messages();
        println!("Message Output: {}", messages);
    }
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

