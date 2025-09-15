use anyhow::{anyhow, Ok};
use core_utils::OrderValue;
use log::info;
use crate::seq::Sequencer;

pub mod seq;

fn get_quote() -> anyhow::Result<String> {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() == 2 {
        let quote=args[1].clone();
        return Ok(quote);
    } else {
        return Err(anyhow!("Only one argument is required"));
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let quote = get_quote()?;
    env_logger::init();
    info!("Starting Sequencer with Quote {quote}");
    let mut sequencer = Sequencer::new(&quote)?;

    let order_value = OrderValue {
        order_id: "ORDER".into(),
        quote: "BTCETH".into(),
        price: 100.10,
        size: 10,
        side: core_utils::Side::ASK,
        order_type: core_utils::OrderType::LIMIT,
    };
    unsafe { sequencer.inbound_manager.as_mut() }
        .unwrap()
        .enqueue(&bincode::serialize(&order_value).unwrap())
        .unwrap();
    sequencer.run()?;
    Ok(())
}
