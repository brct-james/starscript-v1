use itertools::Itertools;
use serde::{Deserialize, Serialize};
use spacetraders::client::Client;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MarketGoodSummary {
    good_symbol: String,
    location_symbol: String,
    volume_per_unit: i32,
    purchase_price_per_unit: i32,
    sell_price_per_unit: i32,
    quantity_available: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_client = spacetraders::client::get_http_client(None);
    let client = Client::new(
        http_client,
        "Greenitthe".to_string(),
        "4be25691-9594-4595-8344-ae3078b4b9fa".to_string(),
    );

    // let info = client.get_my_info().await?;
    // println!("Info: {:?}", info);

    // let oe = client.get_location_info("OE-PM".to_string()).await?;
    // println!("OE: {:?}", oe);

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
    // println!("{}", serde_json::to_string_pretty(&locs_info).unwrap());

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

    // println!("{}", serde_json::to_string_pretty(&mkts).unwrap());
    // println!("{}", serde_json::to_string_pretty(&mkt_symbols).unwrap());
    // println!("{}", serde_json::to_string_pretty(&ships_for_sale).unwrap());

    // Find Potential Routes
    find_routes(0i32, ships_for_sale, locs_info, mkt_symbols); //50i32

    // Calculate Fuel?

    // Score each based on $/time for each ship type (account for cargo size after fuel [and qty available], speed (flight time), price difference, and market share per run [dont want to be bringing 3000 to a station wanting 20 for ex])

    // Future: Find Potential Routes given a ship object, allowing dynamic retasking. Includes travel time to start of proposed route in ranking step, restricted cargo, fuel cost (and whether to fill up on both ends or just one)

    Ok(())
    // spacetraders::client::claim_username(client, "Greenitthe".to_string());
}

fn distance_from_coords(x1: &i32, y1: &i32, x2: &i32, y2: &i32) -> f64 {
    f64::from((x2 - x1).pow(2) + (y2 - y1).pow(2)).sqrt()
}

fn calculate_flight_time(scale: &f64, speed: &f64, distance: &f64) -> f64 {
    let docking_time = 30f64;
    ((distance * scale) / speed) + docking_time
}

fn calculate_fuel_cost(distance: &f64, ship_type: &String, target_type: &String) -> i32 {
    let mut fuel: i32;
    if ship_type == "HM-MK-III" {
        fuel = ((distance / 10f64).round() as i32) + 1;
    } else {
        fuel = ((distance / 7.5f64).round() as i32) + 1;
    }
    if target_type == "Planet" {
        match ship_type.as_str() {
            "MK-MK-III" => fuel += 1,
            "GR-MK-II" => fuel += 3,
            "GR-MK-III" => fuel += 4,
            _ => fuel += 2,
        }
    }
    return fuel;
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct RouteFinancials {
    purchase_price_at_start: i32,
    sell_price_at_end: i32,
    price_delta_per_unit: i32,
    volume_per_unit: i32,
    cargo_units_per_run: i32,
    credits_per_run: i32,
    credits_per_time: i32,
    quantity_at_start: i32,
    quantity_at_end: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct RouteWayfinding {
    start_symbol: String,
    start_type: String,
    start_x: i32,
    start_y: i32,
    end_symbol: String,
    end_type: String,
    end_x: i32,
    end_y: i32,
    distance: i32,
    flight_time: i32,
    fuel_cost_to_end: i32,
    fuel_cost_to_start: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct RouteShipInfo {
    model: String,
    speed: i32,
    load_speed: i32,
    cargo_size: i32,
    cargo_restrictions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Route {
    name: String,
    good: String,
    financials: RouteFinancials,
    wayfinding: RouteWayfinding,
    ship_info: RouteShipInfo,
}

impl Route {
    fn new(
        good: &String,
        ship: &spacetraders::shared::ShipForSale,
        restricted_goods_string: Vec<String>,
        start: &(String, String, i32, i32),
        end: &(String, String, i32, i32),
        goods_summary: &Vec<MarketGoodSummary>,
    ) -> Route {
        let ship_info = RouteShipInfo {
            model: ship.ship_type.clone(),
            speed: ship.speed.clone(),
            load_speed: ship.loading_speed.clone(),
            cargo_size: ship.max_cargo,
            cargo_restrictions: restricted_goods_string,
        };
        let (start_symbol, start_type, start_x, start_y) = start;
        let (end_symbol, end_type, end_x, end_y) = end;
        let distance = distance_from_coords(start_x, start_y, end_x, end_y);
        let flight_time = calculate_flight_time(&3f64, &(ship.speed as f64), &distance);
        let fuel_cost_to_end = calculate_fuel_cost(&distance, &ship.ship_type, &end_type);
        let fuel_cost_to_start = calculate_fuel_cost(&distance, &ship.ship_type, &start_type);
        let wayfinding = RouteWayfinding {
            start_symbol: start_symbol.to_string(),
            start_type: start_type.to_string(),
            start_x: *start_x,
            start_y: *start_y,
            end_symbol: end_symbol.to_string(),
            end_type: end_type.to_string(),
            end_x: *end_x,
            end_y: *end_y,
            distance: distance.round() as i32,
            flight_time: flight_time.round() as i32,
            fuel_cost_to_end: fuel_cost_to_end,
            fuel_cost_to_start: fuel_cost_to_start,
        };
        let price_delta_per_unit =
            goods_summary[1].sell_price_per_unit - goods_summary[0].purchase_price_per_unit;
        let cargo_units_per_run = ship.max_cargo / goods_summary[0].volume_per_unit;
        let credits_per_run = cargo_units_per_run * price_delta_per_unit;
        let financials = RouteFinancials {
            purchase_price_at_start: goods_summary[0].purchase_price_per_unit,
            sell_price_at_end: goods_summary[1].sell_price_per_unit,
            price_delta_per_unit: price_delta_per_unit,
            volume_per_unit: goods_summary[0].volume_per_unit,
            cargo_units_per_run: cargo_units_per_run,
            credits_per_run: credits_per_run,
            credits_per_time: (credits_per_run as f64 / flight_time).round() as i32,
            quantity_at_start: goods_summary[0].quantity_available,
            quantity_at_end: goods_summary[1].quantity_available,
        };
        let name = format!(
            "{} | {} -> {} | {}",
            good.to_string(),
            start_symbol,
            end_symbol,
            ship.ship_type.to_string()
        );
        // Return
        Route {
            name: name,
            good: good.to_string(),
            ship_info: ship_info,
            wayfinding: wayfinding,
            financials: financials,
        }
    }
}

fn find_routes(
    minimum_profit_per_time: i32,
    ships_for_sale: Vec<spacetraders::shared::ShipForSale>,
    locs_info: HashMap<String, (String, String, i32, i32)>,
    mut goods: HashMap<String, Vec<MarketGoodSummary>>,
) -> Vec<Route> {
    let mut routes = Vec::<Route>::new();
    // Filter tradable goods by requiring at least one in-system pair buying/selling each good
    goods.retain(|_, v| (*v).len() >= 2);

    // Collect all start/end permutations for remaining goods
    for (goodname, good) in goods {
        for endpoint_pair in good.into_iter().permutations(2) {
            if endpoint_pair[0].purchase_price_per_unit < endpoint_pair[1].sell_price_per_unit {
                for ship in &ships_for_sale {
                    let restricted = ship
                        .restricted_goods
                        .as_ref()
                        .unwrap_or(&Vec::<::spacetraders::shared::Good>::new())
                        .iter()
                        .map(|&r| r.to_string())
                        .collect::<Vec<String>>();
                    if restricted.len() == 0 || restricted.contains(&goodname) {
                        let new_route = Route::new(
                            &goodname,
                            ship,
                            restricted,
                            &locs_info[&endpoint_pair[0].location_symbol],
                            &locs_info[&endpoint_pair[1].location_symbol],
                            &endpoint_pair,
                        );
                        if new_route.financials.credits_per_time > minimum_profit_per_time {
                            routes.push(new_route);
                        }
                    }
                }
            }
        }
    }

    // Rank routes by various criteria
    let ranked_routes = rank_routes(routes);

    match save_routes(&ranked_routes) {
        Ok(_) => println!("Saved ranked_routes"),
        Err(why) => println!("An error occured while saving ranked_routes: {}", why),
    }

    return ranked_routes;
}

fn rank_routes(routes: Vec<Route>) -> Vec<Route> {
    let mut ranked_routes = routes;
    ranked_routes.sort_by(|a, b| {
        b.financials
            .credits_per_time
            .cmp(&a.financials.credits_per_time)
    });
    return ranked_routes;
}

fn save_routes(routes: &Vec<Route>) -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open("db.json")?;
    // write to file with serde
    serde_json::to_writer_pretty(f, routes)?;

    Ok(())
}
