use axum::{routing::get, Router};
use clap::Parser;
use std::net::SocketAddr;
use tokio::runtime::Runtime;

#[derive(Debug, Parser)]
struct Cli {
    /// The number of tokio instances
    max_tokio_instances: u16,

    /// the factor by which to scale up to the max_tokio_instances
    /// so if max_tokio_instances is 1000 and the scale factor is 10
    /// the simulation will be run as 1 rt -> 10 rt -> 100 rt 1000 rt
    scale_factor: u8,

    /// The number of axum servers per instance of tokio
    servers_per_instance: u16,

    /// true to use axum (webserver). false to just no op every second
    /// axum seems to freak out once you start 255 servers (on linux, untested on windows)
    use_axum: bool,
}

fn main() {
    let cli = Cli::parse();
    println!("running simulation with this cli: {cli:?}");
    println!("========================");

    let mut current_instance_num = 1;

    while current_instance_num <= cli.max_tokio_instances {
        if cli.use_axum {
            println!("running simulation with {current_instance_num} tokio instances and {} axum instance per runtime", cli.servers_per_instance);
        } else {
            println!("running simulation with {current_instance_num} tokio instances and printing \"\" every second");
        }
        let mut current_port_num = 3000_u16;
        let mut rts = vec![];
        for active_runtimes in 0..current_instance_num {
            start_new_runtime_instance(
                cli.servers_per_instance,
                &mut current_port_num,
                &mut rts,
                cli.use_axum,
            );
            println!("current number of active runtime: {}", active_runtimes + 1);
        }

        // let it run for a while for profiling
        std::thread::sleep(std::time::Duration::from_secs(60));

        println!("simulation complete. scaling up instances");
        current_instance_num *= 10;
        println!("------------------------------------------");

        println!("Simulation complete");
    }

    // just dropping everything is fine
}

/// Start a new tokio runtime running inst_info.tasks number of tasks
/// The task will be an axum web server
/// returns a vec of runtimes so nothing gets dropped
fn start_new_runtime_instance(
    tasks_per_instance: u16,
    current_port_num: &mut u16,
    rts: &mut Vec<Runtime>,
    use_axum: bool,
) {
    let rt = Runtime::new().expect("Could not start a new runtime");
    for _ in 0..tasks_per_instance {
        let port_to_use = *current_port_num;

        if use_axum {
            rt.spawn(async move {
                let app = Router::new().route("/", get(root));
                let addr = SocketAddr::from(([127, 0, 0, 1], port_to_use));
                let server = axum::Server::bind(&addr).serve(app.into_make_service());
                server.await.expect("could not start axum server")
            });

            println!("started new axum server on 127.0.0.1:{}", *current_port_num);
            *current_port_num += 1;
        } else {
            rt.spawn(async { something_useless().await });
        }
    }
    rts.push(rt);
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn something_useless() {
    let mut num = 0;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        // not sure if this gets optimized away by compiler so just print nothing to force an op.
        num += 1;
        print!("");
    }
}
