use spacetraders::client::Client;
use std::collections::HashMap;
mod gamemath;
mod traderoutes;
use traderoutes::routes::{
    find_routes, generate_routes_for_ship_and_pick_best_if_meets_minimum_profit_per_time,
    MarketGoodSummary, Route,
};
use traderoutes::steps::{generate_steps, Step, StepSymbol};
mod shipmanager;
mod wayfinding;
use std::thread;
mod shared;
use shared::StarShip;
use wayfinding::StarAtlas;

#[tokio::main]
async fn update_shipmanager_with_api(
    client: &Client,
    shipmanager: &mut shipmanager::ShipManager,
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

    return Ok(shipmanager.clone());
}

#[tokio::main]
async fn _load_shipmanager(
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
    shipmanager.start_route(&tanker.ship.id.unwrap(), &tanker_route);
    let (_, tanker) = shipmanager
        .get_ship(&"ckyabccit124512315s6e0js9gn8".to_string())
        .unwrap();
    // println!("{}", serde_json::to_string_pretty(&tanker).unwrap());
    return Ok((shipmanager, tanker_route, tanker));
}

#[tokio::main]
async fn setup_staratlas(client: &Client) -> Result<StarAtlas, Box<dyn std::error::Error>> {
    let mut staratlas = StarAtlas::new();
    // Get system info
    let systems = client.get_systems_info().await?.systems;
    for system in systems {
        staratlas.add_system(&system.symbol, &system.name, &system.locations);
        // Add marketplace data where available
        for location in system.locations {
            match client
                .get_location_marketplace(&location.symbol.to_string())
                .await
            {
                Ok(mkt) => {
                    staratlas.update_starport_marketplace(
                        &location.symbol,
                        &Some(mkt.marketplace.clone()),
                    );
                    // Now append items to mkt_symbols
                    // let mkt: Vec<spacetraders::shared::MarketplaceData>; (from get_location_marketplace - stored in staratlas)
                    // inserted into mkts hashmap<symbol:String, market:...>
                    for good in mkt.marketplace.iter() {
                        let good_summary = MarketGoodSummary {
                            good_symbol: good.symbol.to_string(),
                            location_symbol: location.symbol.to_string(),
                            volume_per_unit: good.volume_per_unit,
                            purchase_price_per_unit: good.purchase_price_per_unit,
                            sell_price_per_unit: good.sell_price_per_unit,
                            quantity_available: good.quantity_available,
                        };
                        let entry = staratlas
                            .goods
                            .entry(good.symbol.to_string())
                            .or_insert(Vec::<MarketGoodSummary>::new());
                        (*entry).push(good_summary);
                    }
                }
                Err(why) => {
                    println!(
                        "Failed to get marketplace for loc: {}, for reason: {}",
                        location.symbol.to_string(),
                        why
                    );
                    continue;
                }
            };
        }
    }
    return Ok(staratlas);
}

#[tokio::main]
async fn get_ships_for_sale(
    client: &Client,
    system_symbol: &String,
) -> Result<Vec<StarShip>, Box<dyn std::error::Error>> {
    Ok(client
        .get_ships_for_sale(&system_symbol)
        .await?
        .ships
        .iter()
        .map(|s| StarShip::from(s.clone()))
        .collect::<Vec<StarShip>>())
}

#[tokio::main]
async fn setup_shipmanager(
    client: &Client,
) -> Result<shipmanager::ShipManager, Box<dyn std::error::Error>> {
    let my_ships = client.get_my_ships().await?;

    let mut shipmanager = shipmanager::ShipManager::new();
    let _ = shipmanager.load_ships_from_api(
        my_ships
            .ships
            .iter()
            .map(|s| StarShip::from(s.clone()))
            .collect::<Vec<StarShip>>(),
    );

    return Ok(shipmanager);
}

fn _test_generate_way_from_symbols(
    suite: &Vec<(String, String, Vec<String>)>,
    starship: &StarShip,
    staratlas: &StarAtlas,
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

// fn temp_tanker_loop(
//     tanker_route: Route,
//     tanker: shipmanager::ShipStatus,
//     staratlas: StarAtlas,
//     steps: Vec<Step>,
//     client: Client,
//     mut shipmanager: shipmanager::ShipManager,
// ) {
//     // Start Route
//     loop {
//         match update_shipmanager_with_api(&client, shipmanager) {
//             Ok(sm) => shipmanager = sm,
//             Err(why) => panic!("Error updating shipmanager: {}", why),
//         };
//         shipmanager.update_ship_step(&tanker.ship.id.as_ref().unwrap().to_string(), 0);
//         let (_, tanker) = shipmanager
//             .get_ship(&"ckyabccit124512315s6e0js9gn8".to_string())
//             .unwrap();
//         // TODO: unwrap will panic if in transit already!
//         if tanker.ship.location.is_some() {
//             break;
//         } else {
//             println!("Tanker traveling, waiting to start route");
//         }
//     }

//     // Run Route
//     loop {
//         match update_shipmanager_with_api(&client, shipmanager) {
//             Ok(sm) => shipmanager = sm,
//             Err(why) => panic!("Error updating shipmanager: {}", why),
//         };
//         shipmanager.save().unwrap();
//         let (_, tanker) = shipmanager
//             .get_ship(&"ckyabccit124512315s6e0js9gn8".to_string())
//             .unwrap();
//         match (steps[tanker.step.clone().unwrap()].run)(
//             &client.clone(),
//             &tanker.ship.clone(),
//             &staratlas.clone(),
//             &tanker_route.way.clone(),
//             &tanker_route.good.clone(),
//         ) {
//             Ok(delay) => {
//                 thread::sleep(delay);
//                 // println!("Pushing back");
//             }
//             Err(why) => panic!("An error occured while flying start: {}", why),
//         };

//         // Increment Ship Step
//         shipmanager.incr_ship_step(&tanker.ship.id.as_ref().unwrap().to_string());
//         if tanker.step.clone().unwrap() == steps.len() - 1 {
//             // Out of Steps, Route Complete
//             break;
//         }
//     }
// }

async fn handle_ship_routing(
    mut shipmanager: shipmanager::ShipManager,
    ship_id: String,
    client: Client,
    staratlas: StarAtlas,
    steps: Vec<Step>,
) {
    println!("Spawned handle_ship_routing for: {}", ship_id);
    // Run Route
    loop {
        let (_, ship_status) = shipmanager.get_ship(&ship_id).unwrap();
        println!("Run steps for: {}", ship_id);
        // let res = tokio::spawn((steps[ship_status.step.clone().unwrap()].run)(
        //     &client.clone(),
        //     &ship_status.clone(),
        //     &staratlas.clone(),
        // ).await);
        match steps[ship_status.step.clone().unwrap()]
            .run(&client.clone(), &ship_status.clone(), &staratlas.clone())
            .await
        {
            Ok(delay) => {
                println!("Sleep for: {}", ship_id);
                thread::sleep(delay);
                // println!("Pushing back");
            }
            Err(why) => panic!("An error occured while flying start: {}", why),
        };
        println!("Increment step for: {}", ship_id);
        // Increment Ship Step
        shipmanager.incr_ship_step(&ship_status.ship.id.as_ref().unwrap().to_string());
        println!("Check done for: {}", ship_id);
        if ship_status.step.clone().unwrap() == steps.len() - 1 {
            // Out of Steps, Route Complete
            println!("Break for: {}", ship_id);
            break;
        }
    }
}

#[tokio::main]
async fn spawn_ship_routing_handlers(
    shipmanager: shipmanager::ShipManager,
    client: Client,
    staratlas: StarAtlas,
    steps: Vec<Step>,
) {
    // Loop through active and attempt to handle each step
    let mut handles = Vec::new();
    for (_, shipstatus) in shipmanager.active.clone() {
        let ship = shipstatus.ship;
        if ship.location.is_none() {
            // Ship is inactive but travelling, wait till done to start new route
            continue;
        }
        let handle = tokio::spawn(handle_ship_routing(
            shipmanager.clone(),
            ship.id.clone().unwrap(),
            client.clone(),
            staratlas.clone(),
            steps.clone(),
        ));
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let stop_loop = false;

    // Setup Game Client
    let http_client = spacetraders::client::get_http_client(None);
    let client = Client::new(
        http_client,
        "Greenitthe".to_string(),
        "4be25691-9594-4595-8344-ae3078b4b9fa".to_string(),
    );

    // Create Step Definitions
    let steps = generate_steps(vec![
        StepSymbol::TravelStart,
        StepSymbol::BuyGoods,
        StepSymbol::TravelEnd,
        StepSymbol::SellGoods,
    ]);

    // Create staratlas
    let mut staratlas = setup_staratlas(&client).unwrap();
    staratlas.save().unwrap();

    // Get ships for sale
    let mut ships_for_sale = Vec::<StarShip>::new();
    for (symbol, _) in &staratlas.systems {
        match get_ships_for_sale(&client, &symbol) {
            Ok(ships) => ships_for_sale.append(&mut ships.clone()),
            Err(why) => panic!("Error getting ships for sale: {}", why),
        }
    }

    // Create shipmanager
    let shipmanager = setup_shipmanager(&client).unwrap();

    loop {
        // update_staratlas();
        staratlas = setup_staratlas(&client).unwrap();
        staratlas.save().unwrap();
        // update_ships_for_sale();
        // update_shipmanager;
        let mut shipmanager = shipmanager.clone();
        // TODO: COMMENTED OUT TEMPORARILY as currently I am not manually setting ships back to inactive after completion, BUT they should all be inactive before loop completes
        // shipmanager = update_shipmanager_with_api(&client, &mut shipmanager).unwrap();
        // USING setup INSTEAD TEMPORARILY
        shipmanager = setup_shipmanager(&client).unwrap();
        shipmanager.save().unwrap();
        for (_, shipstatus) in shipmanager.inactive.clone() {
            let ship = shipstatus.ship;
            let route: Route;
            let shipid = ship.id.clone().unwrap();
            if ship.location.is_none() {
                // Ship is inactive but travelling, wait till done to start new route
                continue;
            }
            match generate_routes_for_ship_and_pick_best_if_meets_minimum_profit_per_time(
                20i32, &ship, &staratlas,
            ) {
                Some(rte) => {
                    route = rte;
                    println!(
                        "Starting ship (Model: {}, Location: {}, Id: {}) on route: {}",
                        ship.model,
                        ship.location.unwrap(),
                        shipid,
                        route.name
                    );
                }
                _ => {
                    println!(
                        "No valid route found for ship (Model: {}, Location: {}, Id: {})",
                        ship.model,
                        ship.location.unwrap(),
                        shipid
                    );
                    continue;
                }
            };
            // Start Route
            shipmanager.start_route(&shipid, &route);
        }

        // // Find Potential Routes
        // // HashMap is curated routes where ship_type is key
        // // Vec is top routes based on minimum_profit_per_time
        let (_curated_routes, _top_routes) =
            find_routes(50i32, ships_for_sale.clone(), &staratlas.clone());

            
        println!("SPAWN SHIP ROUTING HANDLERS");
        spawn_ship_routing_handlers(
            shipmanager.clone(),
            client.clone(),
            staratlas.clone(),
            steps.clone(),
        );

        if stop_loop {
            break;
        }
        
        // // Take top route, schedule ship to complete once

        // // Score each based on $/time for each ship type (account for cargo size after fuel [and qty available], speed (flight time), price difference, and market share per run [dont want to be bringing 3000 to a station wanting 20 for ex])

        // // Future: Find Potential Routes given a ship object, allowing dynamic retasking. Includes travel time to start of proposed route in ranking step, restricted cargo, fuel cost (and whether to fill up on both ends or just one)

        // let (shipmanager, tanker_route, tanker);
        // match load_shipmanager(&client, curated_routes) {
        //     Ok((res1, res2, res3)) => {
        //         shipmanager = res1;
        //         tanker_route = res2;
        //         tanker = res3;
        //     }
        //     Err(why) => panic!("Error in load_shipmanager: {}", why),
        // };

        // // println!("Test generate_way_from_symbols:");
        // // let suite = vec![
        // //     (
        // //         "OE-PM".to_string(),
        // //         "NA7-TH".to_string(),
        // //         [
        // //             "OE-PM -> OE-W-XV",
        // //             "OE-W-XV -> XV-W-OE",
        // //             "XV-W-OE -> XV-W-ZY1",
        // //             "XV-W-ZY1 -> ZY1-W-XV",
        // //             "ZY1-W-XV -> ZY1-W-NA7",
        // //             "ZY1-W-NA7 -> NA7-W-ZY1",
        // //             "NA7-W-ZY1 -> NA7-TH",
        // //         ]
        // //         .iter()
        // //         .map(|s| s.to_string())
        // //         .collect(),
        // //     ),
        // //     (
        // //         "OE-W-XV".to_string(),
        // //         "NA7-TH".to_string(),
        // //         [
        // //             "OE-W-XV -> XV-W-OE",
        // //             "XV-W-OE -> XV-W-ZY1",
        // //             "XV-W-ZY1 -> ZY1-W-XV",
        // //             "ZY1-W-XV -> ZY1-W-NA7",
        // //             "ZY1-W-NA7 -> NA7-W-ZY1",
        // //             "NA7-W-ZY1 -> NA7-TH",
        // //         ]
        // //         .iter()
        // //         .map(|s| s.to_string())
        // //         .collect(),
        // //     ),
        // //     (
        // //         "OE-PM".to_string(),
        // //         "NA7-W-ZY1".to_string(),
        // //         [
        // //             "OE-PM -> OE-W-XV",
        // //             "OE-W-XV -> XV-W-OE",
        // //             "XV-W-OE -> XV-W-ZY1",
        // //             "XV-W-ZY1 -> ZY1-W-XV",
        // //             "ZY1-W-XV -> ZY1-W-NA7",
        // //             "ZY1-W-NA7 -> NA7-W-ZY1",
        // //         ]
        // //         .iter()
        // //         .map(|s| s.to_string())
        // //         .collect(),
        // //     ),
        // //     (
        // //         "OE-W-XV".to_string(),
        // //         "NA7-W-ZY1".to_string(),
        // //         [
        // //             "OE-W-XV -> XV-W-OE",
        // //             "XV-W-OE -> XV-W-ZY1",
        // //             "XV-W-ZY1 -> ZY1-W-XV",
        // //             "ZY1-W-XV -> ZY1-W-NA7",
        // //             "ZY1-W-NA7 -> NA7-W-ZY1",
        // //         ]
        // //         .iter()
        // //         .map(|s| s.to_string())
        // //         .collect(),
        // //     ),
        // //     (
        // //         "NA7-TH".to_string(),
        // //         "OE-PM".to_string(),
        // //         [
        // //             "NA7-TH -> NA7-W-ZY1",
        // //             "NA7-W-ZY1 -> ZY1-W-NA7",
        // //             "ZY1-W-NA7 -> ZY1-W-XV",
        // //             "ZY1-W-XV -> XV-W-ZY1",
        // //             "XV-W-ZY1 -> XV-W-OE",
        // //             "XV-W-OE -> OE-W-XV",
        // //             "OE-W-XV -> OE-PM",
        // //         ]
        // //         .iter()
        // //         .map(|s| s.to_string())
        // //         .collect(),
        // //     ),
        // //     (
        // //         "NA7-W-ZY1".to_string(),
        // //         "OE-PM".to_string(),
        // //         [
        // //             "NA7-W-ZY1 -> ZY1-W-NA7",
        // //             "ZY1-W-NA7 -> ZY1-W-XV",
        // //             "ZY1-W-XV -> XV-W-ZY1",
        // //             "XV-W-ZY1 -> XV-W-OE",
        // //             "XV-W-OE -> OE-W-XV",
        // //             "OE-W-XV -> OE-PM",
        // //         ]
        // //         .iter()
        // //         .map(|s| s.to_string())
        // //         .collect(),
        // //     ),
        // //     (
        // //         "NA7-TH".to_string(),
        // //         "OE-W-XV".to_string(),
        // //         [
        // //             "NA7-TH -> NA7-W-ZY1",
        // //             "NA7-W-ZY1 -> ZY1-W-NA7",
        // //             "ZY1-W-NA7 -> ZY1-W-XV",
        // //             "ZY1-W-XV -> XV-W-ZY1",
        // //             "XV-W-ZY1 -> XV-W-OE",
        // //             "XV-W-OE -> OE-W-XV",
        // //         ]
        // //         .iter()
        // //         .map(|s| s.to_string())
        // //         .collect(),
        // //     ),
        // //     (
        // //         "NA7-W-ZY1".to_string(),
        // //         "OE-W-XV".to_string(),
        // //         [
        // //             "NA7-W-ZY1 -> ZY1-W-NA7",
        // //             "ZY1-W-NA7 -> ZY1-W-XV",
        // //             "ZY1-W-XV -> XV-W-ZY1",
        // //             "XV-W-ZY1 -> XV-W-OE",
        // //             "XV-W-OE -> OE-W-XV",
        // //         ]
        // //         .iter()
        // //         .map(|s| s.to_string())
        // //         .collect(),
        // //     ),
        // //     (
        // //         "XV-W-OE".to_string(),
        // //         "OE-PM-TR".to_string(),
        // //         ["XV-W-OE -> OE-W-XV", "OE-W-XV -> OE-PM-TR"]
        // //             .iter()
        // //             .map(|s| s.to_string())
        // //             .collect(),
        // //     ),
        // //     (
        // //         "XV-W-OE".to_string(),
        // //         "ZY1-W-XV".to_string(),
        // //         ["XV-W-OE -> XV-W-ZY1", "XV-W-ZY1 -> ZY1-W-XV"]
        // //             .iter()
        // //             .map(|s| s.to_string())
        // //             .collect(),
        // //     ),
        // //     (
        // //         "OE-W-XV".to_string(),
        // //         "XV-W-OE".to_string(),
        // //         ["OE-W-XV -> XV-W-OE"]
        // //             .iter()
        // //             .map(|s| s.to_string())
        // //             .collect(),
        // //     ),
        // //     (
        // //         "OE-W-XV".to_string(),
        // //         "OE-PM-TR".to_string(),
        // //         ["OE-W-XV -> OE-PM-TR"]
        // //             .iter()
        // //             .map(|s| s.to_string())
        // //             .collect(),
        // //     ),
        // // (
        // //         "OE-UC".to_string(),
        // //         "OE-UC-AD".to_string(),
        // //         ["OE-UC -> OE-UC-AD"]
        // //             .iter()
        // //             .map(|s| s.to_string())
        // //             .collect(),
        // //     ),
        // // ];
        // // let test_results = test_generate_way_from_symbols(
        // //     &suite,
        // //     &StarShip::from(tanker.ship.clone()),
        // //     &staratlas.clone(),
        // // );
        // // println!("Test generate_way_from_symbols results:");
        // // for (index, (pass, result)) in test_results.iter().enumerate() {
        // //     let (test_start_loc, test_end_loc, _) = suite[index].clone();
        // //     println!(
        // //         "{} : {} -> {} | {}: {:?}",
        // //         index, test_start_loc, test_end_loc, pass, result
        // //     );
        // // }

        // if tanker_route.financials.credits_per_time <= 40 {
        //     println!(
        //         "Credits per time <= 40: {}",
        //         tanker_route.financials.credits_per_time
        //     );
        //     continue;
        // }

        // temp_tanker_loop(
        //     tanker_route,
        //     tanker,
        //     staratlas,
        //     steps.clone(),
        //     c,
        //     shipmanager,
        // );
    }

    Ok(())
    // spacetraders::client::claim_username(client, "Greenitthe".to_string());
}
