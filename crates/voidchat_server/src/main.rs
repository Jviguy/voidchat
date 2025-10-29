use smol::future::FutureExt;
use std::net::SocketAddr;
use async_net::{TcpListener, TcpStream};
use clap::Parser;
use anyhow::Result;
use async_signal::{Signal, Signals};
use smol::io::AsyncReadExt;
use smol::stream::StreamExt;

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

        let mut signals = Signals::new([
            Signal::Term,
            Signal::Quit,
            Signal::Int,
        ])?.fuse();

        let accept_loop = async {
            loop {
                let (con, addr) = server.accept().await?;
                let handle = smol::spawn(handle_function(con, addr));
                tasks.push(handle);
            }
        };

        let result = accept_loop.or(signals.next()).await;

        match result {
            // `accept_loop` finished (which means it errored)
            futures_lite::future::Either::Left(Err(e)) => {
                eprintln!("Server accept loop failed: {}", e);
            },
            // `signals.next()` finished (we got a signal!)
            futures_lite::future::Either::Right(Some(signal)) => {
                println!("\nReceived signal {}. Shutting down...", signal);
            },
            _ => {
                // This would happen if the accept loop Ok'd or signals stream ended
                println!("\nShutting down...");
            }
        }

        Ok::<(), anyhow::Error>(())
    }).expect("TODO: panic message");
}

async fn handle_function(mut con: TcpStream, addr: SocketAddr) -> Result<()> {
    loop {

    }
}