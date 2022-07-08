use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

use tokio::{net::windows::named_pipe::{ServerOptions, ClientOptions}, io::{AsyncWriteExt, AsyncReadExt}, sync::mpsc};

pub struct Agent {
    counter: Arc<AtomicU64>,
    shutdown_send: mpsc::Sender<()>,
    shutdown_recv: mpsc::Receiver<()>,
}

impl Agent {
    pub const SERVICE_NAME: &'static str = "porcelet-agent";
    pub const SERVICE_DISPLAY_NAME: &'static str = "Porcelet Agent";
    //pub const SERVICE_DESCRIPTION: &'static str = "Porcelet agent manager service.";

    pub const SERVICE_PIPE: &'static str = r"\\.\pipe\porcelet-agent-socket";

    pub fn new() -> Self {
        let (shutdown_send, shutdown_recv) = mpsc::channel(1);
        Self {
            counter: Arc::new(AtomicU64::new(0)),
            shutdown_send,
            shutdown_recv,
        }
    }

    /// Returns a sender for agent shutdown events.
    /// 
    /// It is recommended to use try_send with this, and just pass if the channel
    /// is full or closed because that means a shutdown is already in process.
    pub fn shutdown_sender(&self) -> mpsc::Sender<()> {
        self.shutdown_send.clone()
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(Self::SERVICE_PIPE)?;

        loop {
            tokio::select! {
                // Handle incoming connections:
                connection_result = server.connect() => {
                    match connection_result {
                        Ok(_) => {
                            let counter = self.counter.clone();
                            let mut connected_server = server;
                            server = ServerOptions::new().create(Self::SERVICE_PIPE)?;
                    
                            let _client = tokio::spawn(async move {
                                connected_server.write_u64(counter.fetch_add(1, Ordering::SeqCst)).await?;
                                connected_server.disconnect()?;
                                Ok::<(), std::io::Error>(())
                            });
                        },
                        Err(err) => {
                            log::error!("Named pipe connection error: {}", err);
                        }
                    }
                }

                // Handle shutdown requests:
                _ = self.shutdown_recv.recv() => {
                    self.shutdown_recv.close();
                    let _ = server.disconnect();
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn query_status() -> anyhow::Result<u64> {
        let mut client = ClientOptions::new().open(Self::SERVICE_PIPE)?;
        Ok(client.read_u64().await?)
    }
}
