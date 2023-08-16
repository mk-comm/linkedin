

use chrono::{DateTime, Utc};


pub enum QueueType {
   Connection,
   Message,
   Inmail,
   Withdraw,
}


 
pub struct Queue {
   pub queue_type: QueueType,
   pub api_key: String,
   pub scheduled_at: DateTime<Utc>,
}

impl Queue {
   fn new(queue_type: QueueType, api_key: String, scheduled_at: DateTime<Utc>) -> Self {
      Self {
         queue_type,
         api_key,
         scheduled_at,
      }
   }
}