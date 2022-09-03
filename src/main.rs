mod aquarius_db;

use anyhow::{Ok, Result};

#[async_std::main]
async fn main() -> Result<()> {
    let config = aquarius_db::create_config();

    let mut client = aquarius_db::create_client(config).await?;

    let heats = aquarius_db::get_heats(&mut client).await?;
    println!("Heats count: {}", heats.len());

    let heat_id = heats.get(1).unwrap().id;
    let heat_registrations = aquarius_db::get_heat_registrations(&mut client, heat_id).await?;
    println!("heat_registrations count: {}", heat_registrations.len());

    Ok(())
}
