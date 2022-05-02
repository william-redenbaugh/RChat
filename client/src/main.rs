use std::{alloc::handle_alloc_error};

pub struct MessengerConnection{
    ip: String, 
    port: String, 
    
}

fn new_connection(ip: String, port: String, ){

}

impl MessengerConnection{
    pub fn get_messages(&mut self){

    }

    pub fn send_message(&mut self, msg: String){
        
    }
}

fn main() {
    let is_writing = handle_input(); 

    if is_writing {
        
    }
    else{

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

