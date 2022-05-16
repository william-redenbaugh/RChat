use rusqlite::{params, Connection, Result, MappedRows, NO_PARAMS};

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

pub fn init_message_database(filename_r: String, init_database: String) -> MessageDatabase{
    let mut conn_r = Connection::open_in_memory().unwrap();

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
        let mut request = self.conn.prepare(&req_str); 

        // Error Handling
        match request{
            Err(e)=>{},
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
                    Err(e)=>{}
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
        let mut request = self.conn.prepare(&req_str);
        
        let mut msg_vec = Vec::new(); 
        // Error Handling
        match request{
            Err(e)=>{
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
                    Err(e)=>{}
                };
            }
        }

        return msg_vec;
    }
}