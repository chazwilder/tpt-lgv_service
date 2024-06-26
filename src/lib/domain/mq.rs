use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties, ExchangeKind, BasicProperties};
use serde_json;
use dotenvy::dotenv;
use std::env;
use log::info;
use crate::interfaces::ILgv;


pub async fn plc_to_rabbitmq(lgv: ILgv) -> Result<(), anyhow::Error> {
    dotenv().ok();

    let addr = env::var("RABBITMQ_URL").expect("RABBITMQ_URL must be set");
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    channel
        .exchange_declare(
            "lgv_plc",
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..ExchangeDeclareOptions::default()
            },
            FieldTable::default(),
        )
        .await?;

    info!("Connected to RabbitMQ, exchange declared: lgv_plc");

    let event_json = serde_json::to_string(&lgv)?;

            channel
                .basic_publish(
                    "lgv_plc",
                    "",
                    BasicPublishOptions::default(),
                    event_json.as_bytes(),
                    BasicProperties::default(),
                )
                .await?;

            info!("Published event to RabbitMQ exchange 'lgv_plc': {:?}", &lgv);
    Ok(())
}