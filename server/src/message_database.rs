use rusqlite::{params, Connection, Result, MappedRows, NO_PARAMS};
use std::env; 
use rand::{distributions::Alphanumeric, Rng};

#[derive(Clone)]
pub struct Message{
    pub uuid: i64, 
    pub content: String, 
    pub content_type: String,
    pub sender_username: String, 
    pub unix_timestamp: u32
}

pub struct MessageDatabase{
    conn: Connection,
    filename: String
}

pub fn init_message_database(open_from_memory: bool, filename_r: String, init_database: String) -> MessageDatabase{
    let mut conn_r: Connection; 

    if open_from_memory{
        conn_r = Connection::open_in_memory().unwrap();
    }
    else{
        conn_r = Connection::open(&filename_r).unwrap();
    }

    let mut exe_str = String::from("CREATE TABLE if not exists ");
    exe_str.push_str(&init_database); 
    exe_str.push_str(" (
        uuid              INTEGER PRIMARY KEY,
        content           TEXT NOT NULL,
        content_type      TEXT NOT NULL,
        sender_username   TEXT NOT NULL,
        unix_timestamp    INTEGER
        )");

    conn_r.execute(&exe_str,
        NO_PARAMS,
    ).unwrap();  

    return MessageDatabase{
        conn: conn_r, 
        filename: filename_r
    };
}

impl MessageDatabase{
    pub fn save_message(&mut self, message: Message, message_group: String) -> bool{
        
        let mut exe_str = String::from("INSERT INTO "); 
        exe_str.push_str(&message_group); 
        exe_str.push_str(" (uuid, content, content_type, sender_username, unix_timestamp) VALUES (?1, ?2, ?3, ?4, ?5)");
        self.conn.execute(&exe_str, 
        params![message.uuid, 
        &message.content, 
        &message.content_type, 
        &message.sender_username,
        message.unix_timestamp]).unwrap(); 

        return true; 
    }

    pub fn delete_message(&mut self, uuid: i64, message_group: String) -> bool{
        let mut req_str = String::from("DELETE FROM ");
        req_str.push_str(&message_group); 
        req_str.push_str(" WHERE uuid = "); 
        req_str.push_str(&uuid.to_string());

        let request = self.conn.execute(&req_str, NO_PARAMS);
        match request{
            Ok(_a)=>{
                return true; 
            }
            Err(_e)=>{
                println!("There was a problem deleting the message: {}", _e); 
            }
        }
        return true; 
    }

    pub fn get_message_uuid(&mut self, uuid: i64, message_group: String) -> Message{
        let mut message =  Message{
            uuid: -1, 
            content: String::from(""),
            content_type: String::from(""), 
            sender_username: String::from(""),
            unix_timestamp: 0
        };
        
        let mut req_str = String::from("SELECT uuid, content, content_type, sender_username, unix_timestamp FROM ");
        req_str.push_str(&message_group); 
        req_str.push_str(" WHERE uuid = "); 
        req_str.push_str(&uuid.to_string());
        let request = self.conn.prepare(&req_str); 

        // Error Handling
        match request{
            Err(_e)=>{},
            Ok(mut stmt)=>{
                let msg_iter = stmt.query_map([], |row| {
                    Ok(Message{
                        uuid: row.get(0).unwrap(), 
                        content: row.get(1).unwrap(),
                        content_type: row.get(2).unwrap(), 
                        sender_username: row.get(3).unwrap(),
                        unix_timestamp: row.get(4).unwrap()})
                });
                match msg_iter {
                    Ok(msg_ite)=>{
                        for msg in  msg_ite{
                            message = msg.unwrap(); 
                        }
                    },
                    Err(_e)=>{}
                };
            }
        }

        return message; 
    }

    pub fn get_messages_timestamp(&mut self, timestamp: u32, message_group: String) -> Vec<Message>{        
        
        // Issue and process request...
        let mut req_str = String::from("SELECT uuid, content, content_type, sender_username, unix_timestamp FROM ");
        req_str.push_str(&message_group); 
        req_str.push_str(" WHERE unix_timestamp = "); 
        req_str.push_str(&timestamp.to_string());
        let request = self.conn.prepare(&req_str);
        
        let mut msg_vec = Vec::new(); 
        // Error Handling
        match request{
            Err(_e)=>{
                return msg_vec; 
            },
            Ok(mut stmt)=>{
                // Iterate through SQL matches and return...
                let msg_iter = stmt.query_map([], |row| {
                    Ok(Message{
                        uuid: row.get(0).unwrap(), 
                        content: row.get(1).unwrap(),
                        content_type: row.get(2).unwrap(), 
                        sender_username: row.get(3).unwrap(),
                        unix_timestamp: row.get(4).unwrap()})
                }); 
                match msg_iter {
                    Ok(msg_ite)=>{
                        for msg in  msg_ite{
                            msg_vec.push(msg.unwrap());
                        }
                    },
                    Err(_e)=>{}
                };
            }
        }

        return msg_vec;
    }
}

pub fn _test_cases(){
    println!("Testing Message Database Module..."); 

    env::set_var("RUST_BACKTRACE", "1");

    let mut message_database = init_message_database(
        true, 
        String::from("test.db"),
        String::from("test_conversation"), 
    );

    let mut msg_list = Vec::new(); 
    for x in 0..4096{

        let msg = Message{
            uuid: x as i64,
            content: rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .map(char::from)
                    .collect(), 
            content_type: rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect(),
            sender_username: rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect(), 
            unix_timestamp: x as u32
        };
        
        // Push to saved places. 
        msg_list.push(msg.clone());
        message_database.save_message(msg, String::from("test_conversation"));
    }

    for x in 0..4096{
        let msg_a = &msg_list[x]; 
        let msg_b = &message_database.get_message_uuid(x as i64, String::from("test_conversation")); 

        assert!(msg_a.uuid == msg_b.uuid, "Test case uuid had issues...");
        assert!(msg_a.content.eq(&msg_b.content), "Test case content had issues...");
        assert!(msg_a.content_type.eq(&msg_b.content_type), "Test case content_type had issues...");
        assert!(msg_a.sender_username.eq(&msg_b.sender_username), "Test case sender_username had issues...");
        assert!(msg_a.unix_timestamp == msg_b.unix_timestamp, "Test case unix_timestamp had issues...");
    }


    message_database.delete_message(0, String::from("test_conversation"));
    let msg_b = &message_database.get_message_uuid(0, String::from("test_conversation")); 

    println!("{}", msg_b.content);

    println!("Test cases passed successfully");
}