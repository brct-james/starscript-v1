use spacetraders::client::Client;
use std::collections::HashMap;
mod gamemath;
mod traderoutes;
use traderoutes::routes::{find_routes, MarketGoodSummary, Route};
use traderoutes::steps::{generate_steps, StepSymbol};
mod shipmanager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let http_client = spacetraders::client::get_http_client(None);
    let client = Client::new(
        http_client,
        "Greenitthe".to_string(),
        "4be25691-9594-4595-8344-ae3078b4b9fa".to_string(),
    );

    let systems = client.get_systems_info().await?.systems;
    let oe_locs = &systems
        .into_iter()
        .filter(|sys| sys.symbol == "OE".to_string())
        .collect::<Vec<spacetraders::shared::SystemsInfoData>>()[0]
        .locations;
    let oe_loc_symbols = oe_locs
        .into_iter()
        .map(|loc| &loc.symbol)
        .collect::<Vec<&String>>();

    let mut locs_info = HashMap::new();
    for loc in oe_locs.iter() {
        locs_info.insert(
            loc.symbol.to_string(),
            (
                loc.symbol.to_string(),
                loc.systems_info_type.to_string(),
                loc.x,
                loc.y,
            ),
        );
    }

    let mut mkt_symbols = HashMap::new();
    let mut mkts = HashMap::new();
    let loc_vec = oe_loc_symbols;
    for loc in loc_vec.iter() {
        let mkt = client
            .get_location_marketplace(&loc.to_string())
            .await?
            .marketplace;

        // Now append items to mkt_symbols
        for item in mkt.iter() {
            let item_data = MarketGoodSummary {
                good_symbol: item.symbol.to_string(),
                location_symbol: loc.clone().to_string(),
                volume_per_unit: item.volume_per_unit,
                purchase_price_per_unit: item.purchase_price_per_unit,
                sell_price_per_unit: item.sell_price_per_unit,
                quantity_available: item.quantity_available,
            };
            let entry = mkt_symbols
                .entry(item.symbol.to_string())
                .or_insert(Vec::<MarketGoodSummary>::new());
            (*entry).push(item_data);
        }

        // Lastly append mkt to hashmap
        mkts.insert(loc, mkt);
    }
    let ships_for_sale = client.get_ships_for_sale(&"OE".to_string()).await?.ships;

    // println!("{}", serde_json::to_string_pretty(&ships_for_sale).unwrap());

    // Find Potential Routes
    // HashMap is curated routes where ship_type is key
    // Vec is top routes based on minimum_profit_per_time
    let route: (HashMap<String, Route>, Vec<Route>) =
        find_routes(50i32, ships_for_sale, locs_info, mkt_symbols);

    // Take top route, schedule ship to complete once

    // Score each based on $/time for each ship type (account for cargo size after fuel [and qty available], speed (flight time), price difference, and market share per run [dont want to be bringing 3000 to a station wanting 20 for ex])

    // Future: Find Potential Routes given a ship object, allowing dynamic retasking. Includes travel time to start of proposed route in ranking step, restricted cargo, fuel cost (and whether to fill up on both ends or just one)

    let _steps = generate_steps(vec![
        StepSymbol::BuyFuel,
        StepSymbol::TravelStart,
        StepSymbol::BuyFuel,
        StepSymbol::BuyGoods,
        StepSymbol::TravelEnd,
        StepSymbol::SellGoods,
        StepSymbol::FinishRoute,
    ]);

    let my_ships = client.get_my_ships().await?;
    println!("{}", serde_json::to_string_pretty(&my_ships).unwrap());

    Ok(())
    // spacetraders::client::claim_username(client, "Greenitthe".to_string());
}
