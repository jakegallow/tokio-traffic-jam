use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::runtime::Runtime;

fn main() {
    let one_rt = false;
    let single_rt;
    let mut many_rt: Vec<Runtime> = vec![];

    if one_rt {
        single_rt = Runtime::new().unwrap();
        // start up 4 good servers
        for i in 0..4 {
            let port: u16 = 3000 + (i as u16);
            single_rt.spawn(async move {
                let app = Router::new().route("/", get(root));
                let addr = SocketAddr::from(([127, 0, 0, 1], port.clone()));
                let server = axum::Server::bind(&addr).serve(app.into_make_service());
                server.await.expect("could not start axum server")
            });
            println!("started new axum server on 127.0.0.1:{}", port);
        }

        // set up 1 evil server
        let port: u16 = 3000 + (13 as u16);
        single_rt.spawn(async move {
            let app = Router::new().route("/", get(evil_root));
            let addr = SocketAddr::from(([127, 0, 0, 1], port.clone()));
            let server = axum::Server::bind(&addr).serve(app.into_make_service());
            server.await.expect("could not start axum server")
        });
        println!("started new axum server on 127.0.0.1:{}", port);

    } else {
        // start up 4 good servers
        for i in 0..4 {
            let rt = Runtime::new().unwrap();
            let port: u16 = 3000 + (i as u16);
            rt.spawn(async move {
                let app = Router::new().route("/", get(root));
                let addr = SocketAddr::from(([127, 0, 0, 1], port.clone()));
                let server = axum::Server::bind(&addr).serve(app.into_make_service());
                server.await.expect("could not start axum server")
            });
            println!("started new axum server on 127.0.0.1:{}", port);
            many_rt.push(rt);
        }
        let evil_rt = Runtime::new().unwrap();
        // set up 1 evil server
        let port: u16 = 3000 + (13 as u16);
        evil_rt.spawn(async move {
            let app = Router::new().route("/", get(evil_root));
            let addr = SocketAddr::from(([127, 0, 0, 1], port.clone()));
            let server = axum::Server::bind(&addr).serve(app.into_make_service());
            server.await.expect("could not start axum server")
        });
        println!("started new axum server on 127.0.0.1:{}", port);
        many_rt.push(evil_rt);
    }

    loop {} // just loop until ctrl + c
    //single_rt.shutdown_background();
    for rt in many_rt {
        rt.shutdown_background();
    }
}

/// Start a new tokio runtime running inst_info.tasks number of tasks
/// The task will be an axum web server
/// returns a vec of runtimes so nothing gets dropped
fn start_new_runtime_instance(
    tasks_per_instance: u32,
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
        } 
    }
    rts.push(rt);
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

// basic handler that responds with a static string
async fn evil_root() -> &'static str {
    std::thread::sleep(std::time::Duration::from_secs(30));
    "Hello, World!"
}
