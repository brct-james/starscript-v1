use spacetraders::client::Client;
use std::collections::HashMap;
mod gamemath;
mod traderoutes;
use traderoutes::routes::{find_routes, MarketGoodSummary, Route};
use traderoutes::steps::{generate_steps, StepSymbol};
mod shipmanager;
mod wayfinding;

#[tokio::main]
async fn get_systems_stuff(
    client: &Client,
) -> Result<
    (
        Vec<spacetraders::shared::ShipForSale>,
        HashMap<String, (String, String, i32, i32)>,
        HashMap<String, Vec<MarketGoodSummary>>,
    ),
    Box<dyn std::error::Error>,
> {
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
    return Ok((ships_for_sale, locs_info, mkt_symbols));
}

#[tokio::main]
async fn setup_staratlas(
    client: &Client,
    curated_routes: &HashMap<String, Route>,
) -> Result<
    (
        shipmanager::ShipManager,
        Route,
        shipmanager::ShipStatus,
        wayfinding::StarAtlas,
    ),
    Box<dyn std::error::Error>,
> {
    let my_ships = client.get_my_ships().await?;
    // println!("{}", serde_json::to_string_pretty(&my_ships).unwrap());

    let mut shipmanager = shipmanager::ShipManager::new();
    let _ = shipmanager.load_ships_from_api(my_ships.ships);

    let (_, tanker) = shipmanager
        .get_ship(&"ckyabccit124512315s6e0js9gn8".to_string())
        .unwrap();
    println!("{}", serde_json::to_string_pretty(&tanker).unwrap());
    let tanker_route = curated_routes[&tanker.ship.ship_type].clone();
    println!("{}", serde_json::to_string_pretty(&tanker_route).unwrap());
    shipmanager.start_route(&tanker.ship.id, &tanker_route.name, StepSymbol::BuyFuel);
    let (_, tanker) = shipmanager
        .get_ship(&"ckyabccit124512315s6e0js9gn8".to_string())
        .unwrap();
    println!("{}", serde_json::to_string_pretty(&tanker).unwrap());

    let mut staratlas = wayfinding::StarAtlas::new();
    let oe_system_locations = client
        .get_locations_in_system("OE".to_string())
        .await?
        .locations;
    staratlas.add_system(
        "OE".to_string(),
        "Omicron Eridani".to_string(),
        oe_system_locations,
    );
    println!("{}", serde_json::to_string_pretty(&staratlas).unwrap());
    return Ok((shipmanager, tanker_route, tanker, staratlas));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let http_client = spacetraders::client::get_http_client(None);
    let client = Client::new(
        http_client,
        "Greenitthe".to_string(),
        "4be25691-9594-4595-8344-ae3078b4b9fa".to_string(),
    );

    let (ships_for_sale, locs_info, mkt_symbols);
    match get_systems_stuff(&client) {
        Ok((res1, res2, res3)) => {
            ships_for_sale = res1;
            locs_info = res2;
            mkt_symbols = res3
        }
        Err(why) => panic!("Error getting systems stuff: {}", why),
    };

    // Find Potential Routes
    // HashMap is curated routes where ship_type is key
    // Vec is top routes based on minimum_profit_per_time
    let (curated_routes, _top_routes) = find_routes(50i32, ships_for_sale, locs_info, mkt_symbols);

    // Take top route, schedule ship to complete once

    // Score each based on $/time for each ship type (account for cargo size after fuel [and qty available], speed (flight time), price difference, and market share per run [dont want to be bringing 3000 to a station wanting 20 for ex])

    // Future: Find Potential Routes given a ship object, allowing dynamic retasking. Includes travel time to start of proposed route in ranking step, restricted cargo, fuel cost (and whether to fill up on both ends or just one)

    let steps = generate_steps(vec![
        StepSymbol::BuyFuel,
        StepSymbol::TravelStart,
        StepSymbol::BuyFuel,
        StepSymbol::BuyGoods,
        StepSymbol::TravelEnd,
        StepSymbol::SellGoods,
        StepSymbol::FinishRoute,
    ]);

    let (mut shipmanager, tanker_route, tanker, staratlas);
    match setup_staratlas(&client, &curated_routes) {
        Ok((res1, res2, res3, res4)) => {
            shipmanager = res1;
            tanker_route = res2;
            tanker = res3;
            staratlas = res4;
        }
        Err(why) => panic!("Error getting systems stuff: {}", why),
    };

    let way_to_start = wayfinding::generate_way_to_route_start(
        &tanker_route,
        &tanker.ship.location.as_ref().unwrap().to_string(),
        &tanker.ship,
        &staratlas,
    );
    println!("{}", serde_json::to_string_pretty(&way_to_start).unwrap());

    // tokio::task::spawn(async move {
    //     match (steps[&StepSymbol::BuyFuel].run)(
    //         &client.clone(),
    //         &tanker.ship.clone(),
    //         &staratlas.clone(),
    //         &way_to_start.clone(),
    //     ) {
    //         Ok(_) => println!("Ran BuyFuel"),
    //         Err(why) => println!("An error occured while buying fuel: {}", why),
    //     }
    // });
    match (steps[&tanker.step.clone().unwrap()].run)(
        &client.clone(),
        &tanker.ship.clone(),
        &staratlas.clone(),
        &way_to_start.clone(),
    ) {
        Ok(_) => {
            println!("Cool fuel");
        }
        Err(why) => panic!("An error occured while buying fuel: {}", why),
    };
    shipmanager.update_ship_step(
        &tanker.ship.id.to_string(),
        steps[&tanker.step.clone().unwrap()]
            .next_step_symbol
            .clone(),
    );
    let (_, tanker) = shipmanager
        .get_ship(&"ckyabccit124512315s6e0js9gn8".to_string())
        .unwrap();
    match (steps[&tanker.step.clone().unwrap()].run)(
        &client.clone(),
        &tanker.ship.clone(),
        &staratlas.clone(),
        &tanker_route.wayfinding.clone(),
    ) {
        Ok(_) => {
            println!("Pushing back");
        }
        Err(why) => panic!("An error occured while flying start: {}", why),
    };

    Ok(())
    // spacetraders::client::claim_username(client, "Greenitthe".to_string());
}
