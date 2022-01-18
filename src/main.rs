use spacetraders::client::Client;
use std::collections::HashMap;
mod gamemath;
mod traderoutes;
use traderoutes::routes::{find_routes, MarketGoodSummary, Route};
use traderoutes::steps::{generate_steps, StepSymbol};
mod shipmanager;
mod wayfinding;
use std::thread;
mod shared;
use shared::StarShip;

#[tokio::main]
async fn get_systems_stuff(
    client: &Client,
) -> Result<(Vec<StarShip>, HashMap<String, Vec<MarketGoodSummary>>), Box<dyn std::error::Error>> {
    let systems = client.get_systems_info().await?.systems;
    let oe_locs = &systems
        .clone()
        .into_iter()
        .filter(|sys| sys.symbol == "OE".to_string())
        .collect::<Vec<spacetraders::shared::SystemsInfoData>>()[0]
        .locations;
    let oe_loc_symbols = oe_locs
        .into_iter()
        .map(|loc| &loc.symbol)
        .collect::<Vec<&String>>();
    let xv_locs = &systems
        .clone()
        .into_iter()
        .filter(|sys| sys.symbol == "XV".to_string())
        .collect::<Vec<spacetraders::shared::SystemsInfoData>>()[0]
        .locations;
    let xv_loc_symbols = xv_locs
        .into_iter()
        .map(|loc| &loc.symbol)
        .collect::<Vec<&String>>();
    let zy1_locs = &systems
        .clone()
        .into_iter()
        .filter(|sys| sys.symbol == "ZY1".to_string())
        .collect::<Vec<spacetraders::shared::SystemsInfoData>>()[0]
        .locations;
    let zy1_loc_symbols = zy1_locs
        .into_iter()
        .map(|loc| &loc.symbol)
        .collect::<Vec<&String>>();
    let na7_locs = &systems
        .clone()
        .into_iter()
        .filter(|sys| sys.symbol == "NA7".to_string())
        .collect::<Vec<spacetraders::shared::SystemsInfoData>>()[0]
        .locations;
    let na7_loc_symbols = na7_locs
        .into_iter()
        .map(|loc| &loc.symbol)
        .collect::<Vec<&String>>();

    let ships_for_sale_system = "OE".to_string();
    let locations_symbols = Vec::<&String>::new()
        .into_iter()
        .chain(oe_loc_symbols.into_iter())
        .chain(xv_loc_symbols.into_iter())
        .chain(zy1_loc_symbols.into_iter())
        .chain(na7_loc_symbols.into_iter())
        .collect::<Vec<&String>>();

    let mut mkt_symbols = HashMap::new();
    let mut mkts = HashMap::new();
    let loc_vec = locations_symbols;
    for loc in loc_vec.iter() {
        let mkt: Vec<spacetraders::shared::MarketplaceData>;
        match client.get_location_marketplace(&loc.to_string()).await {
            Ok(locmktplc) => mkt = locmktplc.marketplace,
            Err(why) => {
                println!(
                    "Failed to get marketplace for loc: {}, for reason: {}",
                    loc.to_string(),
                    why
                );
                continue;
            }
        };

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
    let ships_for_sale = client
        .get_ships_for_sale(&ships_for_sale_system)
        .await?
        .ships
        .iter()
        .map(|s| StarShip::from(s.clone()))
        .collect::<Vec<StarShip>>();
    return Ok((ships_for_sale, mkt_symbols));
}

#[tokio::main]
async fn update_shipmanager_with_api(
    client: &Client,
    mut shipmanager: shipmanager::ShipManager,
) -> Result<shipmanager::ShipManager, Box<dyn std::error::Error>> {
    let my_ships = client.get_my_ships().await?;
    // println!("{}", serde_json::to_string_pretty(&my_ships).unwrap());

    let _ = shipmanager.update_ships_from_api(
        my_ships
            .ships
            .iter()
            .map(|s| StarShip::from(s.clone()))
            .collect::<Vec<StarShip>>(),
    );

    return Ok(shipmanager);
}

#[tokio::main]
async fn load_shipmanager(
    client: &Client,
    curated_routes: HashMap<String, Route>,
) -> Result<(shipmanager::ShipManager, Route, shipmanager::ShipStatus), Box<dyn std::error::Error>>
{
    let my_ships = client.get_my_ships().await?;
    // println!("{}", serde_json::to_string_pretty(&my_ships).unwrap());

    let mut shipmanager = shipmanager::ShipManager::new();
    let _ = shipmanager.load_ships_from_api(
        my_ships
            .ships
            .iter()
            .map(|s| StarShip::from(s.clone()))
            .collect::<Vec<StarShip>>(),
    );

    let (_, tanker) = shipmanager
        .get_ship(&"ckyabccit124512315s6e0js9gn8".to_string())
        .unwrap();
    // println!("{}", serde_json::to_string_pretty(&tanker).unwrap());
    let tanker_route = curated_routes[&tanker.ship.model].clone();
    println!("{}", serde_json::to_string_pretty(&tanker_route).unwrap());
    shipmanager.start_route(&tanker.ship.id.unwrap(), &tanker_route.name);
    let (_, tanker) = shipmanager
        .get_ship(&"ckyabccit124512315s6e0js9gn8".to_string())
        .unwrap();
    // println!("{}", serde_json::to_string_pretty(&tanker).unwrap());
    return Ok((shipmanager, tanker_route, tanker));
}

#[tokio::main]
async fn setup_staratlas(
    client: &Client,
) -> Result<wayfinding::StarAtlas, Box<dyn std::error::Error>> {
    let mut staratlas = wayfinding::StarAtlas::new();
    let oe_system_locations = client
        .get_locations_in_system("OE".to_string())
        .await?
        .locations;
    let xv_system_locations = client
        .get_locations_in_system("XV".to_string())
        .await?
        .locations;
    let zy1_system_locations = client
        .get_locations_in_system("ZY1".to_string())
        .await?
        .locations;
    let na7_system_locations = client
        .get_locations_in_system("NA7".to_string())
        .await?
        .locations;
    staratlas.add_system(
        "OE".to_string(),
        "Omicron Eridani".to_string(),
        oe_system_locations,
    );
    staratlas.add_system("XV".to_string(), "Xiav".to_string(), xv_system_locations);
    staratlas.add_system(
        "ZY1".to_string(),
        "Zeon Y1".to_string(),
        zy1_system_locations,
    );
    staratlas.add_system(
        "NA7".to_string(),
        "Niri A7".to_string(),
        na7_system_locations,
    );
    // println!(
    //     "staratlas: {}",
    //     serde_json::to_string_pretty(&staratlas).unwrap()
    // );
    return Ok(staratlas);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let stop_loop = false;
    loop {
        let http_client = spacetraders::client::get_http_client(None);
        let client = Client::new(
            http_client,
            "Greenitthe".to_string(),
            "4be25691-9594-4595-8344-ae3078b4b9fa".to_string(),
        );

        let (ships_for_sale, mkt_symbols);
        match get_systems_stuff(&client) {
            Ok((res1, res2)) => {
                ships_for_sale = res1;
                mkt_symbols = res2;
            }
            Err(why) => panic!("Error getting systems stuff: {}", why),
        };

        let steps = generate_steps(vec![
            StepSymbol::TravelStart,
            StepSymbol::BuyGoods,
            StepSymbol::TravelEnd,
            StepSymbol::SellGoods,
            StepSymbol::FinishRoute,
        ]);

        let staratlas;
        match setup_staratlas(&client) {
            Ok(res1) => {
                staratlas = res1;
            }
            Err(why) => panic!("Error in setup_staratlas: {}", why),
        };

        // Find Potential Routes
        // HashMap is curated routes where ship_type is key
        // Vec is top routes based on minimum_profit_per_time
        let (curated_routes, _top_routes) =
            find_routes(50i32, ships_for_sale, mkt_symbols, &staratlas);

        // Take top route, schedule ship to complete once

        // Score each based on $/time for each ship type (account for cargo size after fuel [and qty available], speed (flight time), price difference, and market share per run [dont want to be bringing 3000 to a station wanting 20 for ex])

        // Future: Find Potential Routes given a ship object, allowing dynamic retasking. Includes travel time to start of proposed route in ranking step, restricted cargo, fuel cost (and whether to fill up on both ends or just one)

        let (shipmanager, tanker_route, tanker);
        match load_shipmanager(&client, curated_routes) {
            Ok((res1, res2, res3)) => {
                shipmanager = res1;
                tanker_route = res2;
                tanker = res3;
            }
            Err(why) => panic!("Error in load_shipmanager: {}", why),
        };

        // println!("Test generate_way_from_symbols:");
        // let suite = vec![
        //     (
        //         "OE-PM".to_string(),
        //         "NA7-TH".to_string(),
        //         [
        //             "OE-PM -> OE-W-XV",
        //             "OE-W-XV -> XV-W-OE",
        //             "XV-W-OE -> XV-W-ZY1",
        //             "XV-W-ZY1 -> ZY1-W-XV",
        //             "ZY1-W-XV -> ZY1-W-NA7",
        //             "ZY1-W-NA7 -> NA7-W-ZY1",
        //             "NA7-W-ZY1 -> NA7-TH",
        //         ]
        //         .iter()
        //         .map(|s| s.to_string())
        //         .collect(),
        //     ),
        //     (
        //         "OE-W-XV".to_string(),
        //         "NA7-TH".to_string(),
        //         [
        //             "OE-W-XV -> XV-W-OE",
        //             "XV-W-OE -> XV-W-ZY1",
        //             "XV-W-ZY1 -> ZY1-W-XV",
        //             "ZY1-W-XV -> ZY1-W-NA7",
        //             "ZY1-W-NA7 -> NA7-W-ZY1",
        //             "NA7-W-ZY1 -> NA7-TH",
        //         ]
        //         .iter()
        //         .map(|s| s.to_string())
        //         .collect(),
        //     ),
        //     (
        //         "OE-PM".to_string(),
        //         "NA7-W-ZY1".to_string(),
        //         [
        //             "OE-PM -> OE-W-XV",
        //             "OE-W-XV -> XV-W-OE",
        //             "XV-W-OE -> XV-W-ZY1",
        //             "XV-W-ZY1 -> ZY1-W-XV",
        //             "ZY1-W-XV -> ZY1-W-NA7",
        //             "ZY1-W-NA7 -> NA7-W-ZY1",
        //         ]
        //         .iter()
        //         .map(|s| s.to_string())
        //         .collect(),
        //     ),
        //     (
        //         "OE-W-XV".to_string(),
        //         "NA7-W-ZY1".to_string(),
        //         [
        //             "OE-W-XV -> XV-W-OE",
        //             "XV-W-OE -> XV-W-ZY1",
        //             "XV-W-ZY1 -> ZY1-W-XV",
        //             "ZY1-W-XV -> ZY1-W-NA7",
        //             "ZY1-W-NA7 -> NA7-W-ZY1",
        //         ]
        //         .iter()
        //         .map(|s| s.to_string())
        //         .collect(),
        //     ),
        //     (
        //         "NA7-TH".to_string(),
        //         "OE-PM".to_string(),
        //         [
        //             "NA7-TH -> NA7-W-ZY1",
        //             "NA7-W-ZY1 -> ZY1-W-NA7",
        //             "ZY1-W-NA7 -> ZY1-W-XV",
        //             "ZY1-W-XV -> XV-W-ZY1",
        //             "XV-W-ZY1 -> XV-W-OE",
        //             "XV-W-OE -> OE-W-XV",
        //             "OE-W-XV -> OE-PM",
        //         ]
        //         .iter()
        //         .map(|s| s.to_string())
        //         .collect(),
        //     ),
        //     (
        //         "NA7-W-ZY1".to_string(),
        //         "OE-PM".to_string(),
        //         [
        //             "NA7-W-ZY1 -> ZY1-W-NA7",
        //             "ZY1-W-NA7 -> ZY1-W-XV",
        //             "ZY1-W-XV -> XV-W-ZY1",
        //             "XV-W-ZY1 -> XV-W-OE",
        //             "XV-W-OE -> OE-W-XV",
        //             "OE-W-XV -> OE-PM",
        //         ]
        //         .iter()
        //         .map(|s| s.to_string())
        //         .collect(),
        //     ),
        //     (
        //         "NA7-TH".to_string(),
        //         "OE-W-XV".to_string(),
        //         [
        //             "NA7-TH -> NA7-W-ZY1",
        //             "NA7-W-ZY1 -> ZY1-W-NA7",
        //             "ZY1-W-NA7 -> ZY1-W-XV",
        //             "ZY1-W-XV -> XV-W-ZY1",
        //             "XV-W-ZY1 -> XV-W-OE",
        //             "XV-W-OE -> OE-W-XV",
        //         ]
        //         .iter()
        //         .map(|s| s.to_string())
        //         .collect(),
        //     ),
        //     (
        //         "NA7-W-ZY1".to_string(),
        //         "OE-W-XV".to_string(),
        //         [
        //             "NA7-W-ZY1 -> ZY1-W-NA7",
        //             "ZY1-W-NA7 -> ZY1-W-XV",
        //             "ZY1-W-XV -> XV-W-ZY1",
        //             "XV-W-ZY1 -> XV-W-OE",
        //             "XV-W-OE -> OE-W-XV",
        //         ]
        //         .iter()
        //         .map(|s| s.to_string())
        //         .collect(),
        //     ),
        //     (
        //         "XV-W-OE".to_string(),
        //         "OE-PM-TR".to_string(),
        //         ["XV-W-OE -> OE-W-XV", "OE-W-XV -> OE-PM-TR"]
        //             .iter()
        //             .map(|s| s.to_string())
        //             .collect(),
        //     ),
        //     (
        //         "XV-W-OE".to_string(),
        //         "ZY1-W-XV".to_string(),
        //         ["XV-W-OE -> XV-W-ZY1", "XV-W-ZY1 -> ZY1-W-XV"]
        //             .iter()
        //             .map(|s| s.to_string())
        //             .collect(),
        //     ),
        //     (
        //         "OE-W-XV".to_string(),
        //         "XV-W-OE".to_string(),
        //         ["OE-W-XV -> XV-W-OE"]
        //             .iter()
        //             .map(|s| s.to_string())
        //             .collect(),
        //     ),
        //     (
        //         "OE-W-XV".to_string(),
        //         "OE-PM-TR".to_string(),
        //         ["OE-W-XV -> OE-PM-TR"]
        //             .iter()
        //             .map(|s| s.to_string())
        //             .collect(),
        //     ),
        // (
        //         "OE-UC".to_string(),
        //         "OE-UC-AD".to_string(),
        //         ["OE-UC -> OE-UC-AD"]
        //             .iter()
        //             .map(|s| s.to_string())
        //             .collect(),
        //     ),
        // ];
        // let test_results = test_generate_way_from_symbols(
        //     &suite,
        //     &StarShip::from(tanker.ship.clone()),
        //     &staratlas.clone(),
        // );
        // println!("Test generate_way_from_symbols results:");
        // for (index, (pass, result)) in test_results.iter().enumerate() {
        //     let (test_start_loc, test_end_loc, _) = suite[index].clone();
        //     println!(
        //         "{} : {} -> {} | {}: {:?}",
        //         index, test_start_loc, test_end_loc, pass, result
        //     );
        // }

        if tanker_route.financials.credits_per_time <= 40 {
            println!(
                "Credits per time <= 40: {}",
                tanker_route.financials.credits_per_time
            );
            continue;
        }

        temp_tanker_loop(tanker_route, tanker, staratlas, steps, client, shipmanager);

        if stop_loop {
            break;
        }
    }

    Ok(())
    // spacetraders::client::claim_username(client, "Greenitthe".to_string());
}

fn _test_generate_way_from_symbols(
    suite: &Vec<(String, String, Vec<String>)>,
    starship: &StarShip,
    staratlas: &wayfinding::StarAtlas,
) -> Vec<(bool, Vec<String>)> {
    let mut results = Vec::<(bool, Vec<String>)>::new();
    for (start, end, assertion) in suite {
        let way = wayfinding::generate_way_from_symbols(start, end, starship, staratlas);
        let result = way
            .legs
            .iter()
            .map(|l| format!("{} -> {}", l.start_symbol, l.end_symbol))
            .collect::<Vec<String>>();
        results.push((result == *assertion, result));
    }
    return results;
}

fn temp_tanker_loop(
    tanker_route: Route,
    tanker: shipmanager::ShipStatus,
    staratlas: wayfinding::StarAtlas,
    steps: Vec<traderoutes::steps::Step>,
    client: Client,
    mut shipmanager: shipmanager::ShipManager,
) {
    // Start Route
    loop {
        match update_shipmanager_with_api(&client, shipmanager) {
            Ok(sm) => shipmanager = sm,
            Err(why) => panic!("Error updating shipmanager: {}", why),
        };
        shipmanager.update_ship_step(&tanker.ship.id.as_ref().unwrap().to_string(), 0);
        let (_, tanker) = shipmanager
            .get_ship(&"ckyabccit124512315s6e0js9gn8".to_string())
            .unwrap();
        // TODO: unwrap will panic if in transit already!
        if tanker.ship.location.is_some() {
            break;
        } else {
            println!("Tanker traveling, waiting to start route");
        }
    }

    // Run Route
    loop {
        match update_shipmanager_with_api(&client, shipmanager) {
            Ok(sm) => shipmanager = sm,
            Err(why) => panic!("Error updating shipmanager: {}", why),
        };
        match shipmanager.save() {
            Ok(_) => println!("Saved shipmanager to file"),
            Err(why) => panic!("Couldnt save shipmanager due to error: {}", why),
        };
        let (_, tanker) = shipmanager
            .get_ship(&"ckyabccit124512315s6e0js9gn8".to_string())
            .unwrap();
        match (steps[tanker.step.clone().unwrap()].run)(
            &client.clone(),
            &tanker.ship.clone(),
            &staratlas.clone(),
            &tanker_route.wayfinding.clone(),
            &tanker_route.good.clone(),
        ) {
            Ok(delay) => {
                thread::sleep(delay);
                // println!("Pushing back");
            }
            Err(why) => panic!("An error occured while flying start: {}", why),
        };

        // Increment Ship Step
        shipmanager.incr_ship_step(&tanker.ship.id.as_ref().unwrap().to_string());
        if tanker.step.clone().unwrap() == steps.len() - 1 {
            // Out of Steps, Route Complete
            break;
        }
    }
}
