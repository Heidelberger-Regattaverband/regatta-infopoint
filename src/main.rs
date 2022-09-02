mod aquarius_db;

use anyhow::{Ok, Result};

#[async_std::main]
async fn main() -> Result<()> {
    let config = aquarius_db::create_config();

    let mut client = aquarius_db::create_client(config).await?;

    let heats = aquarius_db::get_heats(&mut client).await?;
    println!("Heats count: {}", heats.len());

    let heat_id = heats.first().unwrap().id;
    let heat = aquarius_db::get_heat_result(&mut client, heat_id).await?;
    aquarius_db::print_heat(&heat);

    Ok(())
}
