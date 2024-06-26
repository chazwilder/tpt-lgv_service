use std::env;
use lgv_service_lib::domain::{get_lgv_values, load_ips, plc_to_rabbitmq};
use log::{error, info};
use log4rs;
use warp::Filter;

async fn run_tasks() -> Result<(), anyhow::Error> {
    loop {
        let lgv_ips = load_ips().await;
        let mut tasks = Vec::new();

        for ip in lgv_ips {
            let ip_clone = ip.clone();
            let task = tokio::spawn(async move {
                match get_lgv_values(&ip_clone).await {
                    Ok(record) => {
                        info!("Sending record to MQ for {:?}", &record);
                        if let Err(err) = plc_to_rabbitmq(record.clone()).await {
                            error!("Error sending to MQ: {:?}", err);
                        } else {
                            info!("Record for LGV {:?} sent to MQ successfully", record);
                        }
                    }
                    Err(e) => {
                        error!("Failed to get LGV values for {}: {:?}", ip_clone, e);
                    }
                }
            });
            tasks.push(task);
        }

        for task in tasks {
            if let Err(e) = task.await {
                error!("Task failed: {:?}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env::set_var("RUST_BACKTRACE", "0");
    log4rs::init_file("C:\\Users\\cwilder\\Desktop\\dev\\TPT\\lgv_service\\log4rs.yaml", Default::default())
        .expect("Failed to initialize logger from config file");
    info!("lgv_service started with heartbeat");

    let health_route = warp::path("heartbeat").map(|| "OK");
    let server = tokio::spawn(warp::serve(health_route).run(([0, 0, 0, 0], 3035)));

    tokio::select! {
        result = run_tasks() => {
            if let Err(e) = result {
                error!("Error running tasks: {:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down");
        }
    }

    server.abort();
    println!("Shutting down");
    Ok(())
}
