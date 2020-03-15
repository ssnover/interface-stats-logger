use std::{thread, time};
use futures::stream::TryStreamExt;
use rtnetlink::packet::rtnl::link::nlas::{Nla, Stats64, Stats64Buffer};
use netlink_packet_utils::traits::Parseable;
use rusqlite::params;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const INTERFACE_SAMPLE_PERIOD_SECONDS: u64 = 30;
const DATABASE_PATH: &str = "/home/ssnover/develop/interface-stats-logger/db/if-stats.db";
const INTERFACE_LIST: [&str; 1]  = ["wlp2s0"];

#[tokio::main]
async fn main() -> Result<(), ()> {
    // daemonize - this can wait
    // set handler for sigint
    // read config file
    // initialize database if it does not exist,
    // establish connection to database
    // periodically check network data and log it

    let _ = daemonize::Daemonize::new()
        .pid_file("/tmp/interface-stats-logger.pid")
        .working_directory("/tmp")
        .stdout(std::fs::File::create("/tmp/test.out").unwrap())
        .stderr(std::fs::File::create("/tmp/test.err").unwrap())
        .exit_action(|| println!("Daemonizing..."))
        .start();

    let done = Arc::new(AtomicBool::new(false));
    let _ = signal_hook::flag::register(libc::SIGINT, Arc::clone(&done));
    let _ = signal_hook::flag::register(libc::SIGTERM, Arc::clone(&done));

    let (connection, nl_handle, _) = rtnetlink::new_connection().unwrap();
    tokio::spawn(connection);
    let db_connection = rusqlite::Connection::open(DATABASE_PATH).unwrap();

    println!("Periodically checking network interfaces");
    while !done.load(Ordering::SeqCst) {
        let now = time::Instant::now();
        if let Err(e) = check_network_interfaces(&nl_handle, &db_connection).await {
            eprintln!("Encountered an error parsing network metadata: {}", e);
            break;
        }

        thread::sleep(time::Duration::from_secs(INTERFACE_SAMPLE_PERIOD_SECONDS) - now.elapsed());
    }
    
    Ok(())
}

async fn check_network_interfaces(nl_handle: &rtnetlink::Handle, db_conn: &rusqlite::Connection) -> Result<(), rtnetlink::Error>
{
    for link_name in &INTERFACE_LIST {
        let mut nl_results = nl_handle.link().get().set_name_filter(link_name.to_string()).execute();
        if let Some(response) = nl_results.try_next().await? {
            for attribute in response.nlas.into_iter() {
                if let Nla::Stats64(buffer) = attribute {
                    let buffer = Stats64Buffer::new(buffer);
                    let stats = Stats64::parse(&buffer).unwrap();

                    println!("{} stats: RX bytes {}, TX bytes {}", &link_name, stats.rx_bytes, stats.tx_bytes);
                    db_conn.execute("INSERT INTO InterfaceMetadata (InterfaceName, ReceivedBytes, TransmittedBytes) VALUES (?001, ?002, ?003)",
                        params![&link_name, stats.rx_bytes as i64, stats.tx_bytes as i64],).unwrap();
                }
            }
        }
    }

    Ok(())
}