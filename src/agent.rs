use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

use tokio::{net::windows::named_pipe::{ServerOptions, ClientOptions}, io::{AsyncWriteExt, AsyncReadExt}};

pub struct Agent {
    counter: Arc<AtomicU64>,
}

impl Agent {
    pub const SERVICE_NAME: &'static str = "porcelet-agent";
    pub const SERVICE_DISPLAY_NAME: &'static str = "Porcelet Agent";
    //pub const SERVICE_DESCRIPTION: &'static str = "Porcelet agent manager service.";

    pub const SERVICE_PIPE: &'static str = r"\\.\pipe\porcelet-agent-socket";

    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(Self::SERVICE_PIPE)?;

        loop {
            server.connect().await?;
            let mut connected_server = server;
            let counter = self.counter.clone();
            server = ServerOptions::new().create(Self::SERVICE_PIPE)?;
    
            let _client = tokio::spawn(async move {
                connected_server.write_u64(counter.fetch_add(1, Ordering::SeqCst)).await?;
                connected_server.disconnect()?;
                Ok::<(), std::io::Error>(())
            });
        }
    }

    pub async fn query_status() -> anyhow::Result<u64> {
        let mut client = ClientOptions::new().open(Self::SERVICE_PIPE)?;
        Ok(client.read_u64().await?)
    }
}
