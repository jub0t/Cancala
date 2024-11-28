use std::pin::Pin;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use super::broadcast::broadcast_service_server::BroadcastService;
use super::broadcast::{BroadcastMessage, Empty};
use crate::bot::manager::SharedBotManager;

// Define the stream type for the broadcast service
type ResponseStream = Pin<Box<dyn Stream<Item = Result<BroadcastMessage, Status>> + Send>>;

pub struct MyBroadcastService {
    pub bot_manager: SharedBotManager, // Add this field
}

impl MyBroadcastService {
    // Modify the constructor to accept a bot manager
    pub fn new(bot_manager: SharedBotManager) -> Self {
        MyBroadcastService { bot_manager }
    }
}

#[tonic::async_trait]
impl BroadcastService for MyBroadcastService {
    type SubscribeStream = ResponseStream;

    async fn subscribe(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let (tx, rx) = channel(4); // Buffered channel with capacity of 4

        // Lock the mutex to access the BotManager and its stdout_receiver
        let manager = self
            .bot_manager
            .lock()
            .map_err(|_| Status::internal("Failed to lock bot manager"))?;

        // Clone the receiver
        let receiver_clone = manager.stdout_receiver.clone();

        // Spawn a new task to handle receiving messages
        let tx = tx.clone(); // Clone the sender to move it into the task
        tokio::spawn(async move {
            loop {
                match receiver_clone.as_ref().lock().unwrap().recv() {
                    Err(e) => {
                        println!("{:#?}", e)
                    }

                    Ok(io) => {
                        // TODO: implement proper broadcasting back to the listener.
                        println!("[DEBUG] I/O Data: {:#?}", io.data);
                        // io.data = Vec<u8>

                        BroadcastMessage::default();
                        tx.send(Ok(BroadcastMessage {
                            message: String::from_utf8_lossy(&io.data).to_string(),
                        }));
                    }
                };
            }
        });

        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream) as Self::SubscribeStream))
    }
}