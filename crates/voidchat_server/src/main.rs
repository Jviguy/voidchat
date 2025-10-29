use anyhow::Result;
use async_net::{TcpListener, TcpStream};
use async_signal::{Signal, Signals};
use clap::Parser;
use smol::future::FutureExt;
use smol::stream::StreamExt;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to run the server on
    #[arg(short, long, env = "VOID_PORT", default_value = "1444")]
    port: u16,

    /// Password for the void.
    #[arg(short, long, env = "VOID_PASSWORD")]
    password: Option<String>,
}

fn main() {
    let config = Args::parse();
    let hash = config.password.map(voidchat_proto::hash_password);
    smol::block_on(async {
        // establish the actual server.
        let server = TcpListener::bind(("0.0.0.0", config.port)).await?;

        // holds the handles of all the tasks we are running.
        let mut tasks = vec![];

        let mut signals = Signals::new([Signal::Term, Signal::Quit, Signal::Int])?
            .fuse()
            .next();

        let accept_loop = async {
            loop {
                let (con, addr) = server.accept().await?;
                let handle = smol::spawn(handle_function(con, addr));
                tasks.push(handle);
            }
            Ok::<(), anyhow::Error>(())
        };

        let winner = accept_loop.or(signals).await;

        Ok::<(), anyhow::Error>(())
    })
    .expect("TODO: panic message");
}

async fn handle_function(mut con: TcpStream, addr: SocketAddr) -> Result<()> {
    loop {}
}
