use std::{io::{self,  Write}, net::{IpAddr, Ipv4Addr}, sync::mpsc::{channel, Sender}};
use tokio::{net::TcpStream, task};
use bpaf::Bpaf;

// max port
const MAX: u16 = 65535;

// address fallback
const IPFALLBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1 ));

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]

//CLI arguments
pub struct Arguments {
    // Address argument
    #[bpaf(long, short, argument("Address"), fallback(IPFALLBACK))]
    /// The address you want to sniff. Must be a valid ipv4 address. Falls back to 127.0.0.1
    pub address: IpAddr,
    #[bpaf(
        long("start"),
        short('s'),
        fallback(1u16),
        guard(start_port_guard, "start port must be greater than 0"))]
    /// The port you want to start scanning from. Must be greater than 0. Falls back to 1
    pub start_port: u16,
    #[bpaf(
        long("end"),
        short('e'),
        fallback(MAX),
        guard(end_port_guard, "end port must be less than 65535"))]
    /// The port you want to end scanning at. Must be less than 65535. Falls back to 65535
    pub end_port: u16,
}

fn start_port_guard(input: &u16) -> bool {
    *input > 0
}

fn end_port_guard(input: &u16) -> bool {
    *input <= MAX
}

// scan the port
async fn scan(tx: Sender<u16>, port: u16, addr: IpAddr) {
    // attempt to connect to the port
    match TcpStream::connect(format!("{}:{}", addr, port)).await {
        // if the connection is successful, send the port number
        Ok(_) => {
            print!(".");
            io::stdout().flush().unwrap();
            tx.send(port).unwrap();
        }
        // if the connection is unsuccessful, do nothing
        Err(_) => {}
    }
}

#[tokio::main]
async fn main() {
    // collect the arguments
    let opts: Arguments = arguments().run();
    // create a channel to send the open ports
    let(tx, rx) = channel();
    // iterate through all the ports to spawn a single task for each
    for i in opts.start_port..=opts.end_port {
        let tx = tx.clone();
        task::spawn(async move { scan(tx, i, opts.address).await });
    }
    // create a vector to store the open ports
    let mut out = vec![];
    // drop the tx clone
    drop(tx);

    // iterate through the open ports and push them to the vector
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();
    for v in out {
        // print the open ports
        println!("{} is open", v);
    }
}