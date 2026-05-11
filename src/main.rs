use borsh::{BorshDeserialize, BorshSerialize};
use crosstown_bus::{CrosstownBus, MessageHandler, HandleError};
use std::{thread, time};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UserCreatedEventMessage {
    pub user_id: String,
    pub user_name: String
}

pub struct UserCreatedHandler;

impl MessageHandler<UserCreatedEventMessage> for UserCreatedHandler {

    fn handle(&self, message: Box<UserCreatedEventMessage>
    ) -> Result<(), HandleError> {
        let ten_millis = time::Duration::from_millis(1000);
        let now = time::Instant::now();

        thread::sleep(ten_millis);
        println!("In Aldo's Computer [NPM: 2406423055]. Message received: {:?}", message);

        Ok(())
    }
    
    fn get_handler_action(&self) -> String {
        "user_created_handler".to_string()
    }
}

fn main() {
    dotenv::dotenv().ok();
    let amqp_url = std::env::var("AMQP_URL").unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".into());
    println!("Connecting to RabbitMQ Broker...");
    let listener = CrosstownBus::new_queue_listener(amqp_url).unwrap();
    println!("Connected successfully! Listening for messages...");
    
    _ = listener.listen("user_created".to_owned(), UserCreatedHandler{}, 
    crosstown_bus::QueueProperties { auto_delete: false, durable: true,
    use_dead_letter: true });

    loop {
    }
}