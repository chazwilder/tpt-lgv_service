use lgv_service_lib::domain::get_lgv_values;
use lgv_service_lib::domain::load_ips;
use lgv_service_lib::domain::plc_to_rabbitmq;
use log;
use log4rs;
use log::{error, info};
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    log4rs::init_file("C:\\Users\\cwilder\\Desktop\\dev\\TPT\\lgv_service\\log4rs.yaml", Default::default())
    .expect("Failed to initialize logger from config file");
    info!("lgv_service started with heartbeat");
    let health_route = warp::path("heartbeat").map(|| "OK");
    let server = tokio::spawn(warp::serve(health_route).run(([0, 0, 0, 0], 3035)));
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

    tokio::signal::ctrl_c().await?;
    server.abort();
    println!("Shutting down");
    Ok(())
}
