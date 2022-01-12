use super::super::gamemath;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MarketGoodSummary {
    pub good_symbol: String,
    pub location_symbol: String,
    pub volume_per_unit: i32,
    pub purchase_price_per_unit: i32,
    pub sell_price_per_unit: i32,
    pub quantity_available: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct RouteFinancials {
    pub purchase_price_at_start: i32,
    pub sell_price_at_end: i32,
    pub price_delta_per_unit: i32,
    pub volume_per_unit: i32,
    pub cargo_units_per_run: i32,
    pub credits_per_run: i32,
    pub credits_per_time: i32,
    pub quantity_at_start: i32,
    pub quantity_at_end: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct RouteWayfinding {
    pub start_symbol: String,
    pub start_type: String,
    pub start_x: i32,
    pub start_y: i32,
    pub end_symbol: String,
    pub end_type: String,
    pub end_x: i32,
    pub end_y: i32,
    pub distance: i32,
    pub flight_time: i32,
    pub fuel_cost_to_end: i32,
    pub fuel_cost_to_start: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct RouteShipInfo {
    pub model: String,
    pub speed: i32,
    pub load_speed: i32,
    pub cargo_size: i32,
    pub cargo_restrictions: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Route {
    pub name: String,
    pub good: String,
    pub financials: RouteFinancials,
    pub wayfinding: RouteWayfinding,
    pub ship_info: RouteShipInfo,
}

impl Route {
    pub fn new(
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
        let distance = gamemath::distance_from_coords(start_x, start_y, end_x, end_y);
        let flight_time = gamemath::calculate_flight_time(&3f64, &(ship.speed as f64), &distance);
        let fuel_cost_to_end = gamemath::calculate_fuel_cost(&distance, &ship.ship_type, &end_type);
        let fuel_cost_to_start =
            gamemath::calculate_fuel_cost(&distance, &ship.ship_type, &start_type);
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

pub fn find_routes(
    minimum_profit_per_time: i32,
    ships_for_sale: Vec<spacetraders::shared::ShipForSale>,
    locs_info: HashMap<String, (String, String, i32, i32)>,
    mut goods: HashMap<String, Vec<MarketGoodSummary>>,
) -> (HashMap<String, Route>, Vec<Route>) {
    let mut routes = Vec::<Route>::new();
    // Filter tradable goods by requiring at least one in-system pair buying/selling each good
    goods.retain(|_, v| (*v).len() >= 2);
    let mut ship_models = HashSet::<String>::new();
    // Collect all start/end permutations for remaining goods
    for (goodname, good) in goods {
        for endpoint_pair in good.into_iter().permutations(2) {
            for ship in &ships_for_sale {
                ship_models.insert(ship.ship_type.to_string());
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
                    routes.push(new_route);
                }
            }
        }
    }

    // Rank routes by best w.r.t. credits/time
    let ranked_routes = rank_routes(routes);

    // Curate routes by best w.r.t. ship type in credits/time
    let routes_curated = curate_routes(ranked_routes.clone(), Vec::from_iter(ship_models));

    // Get top routes regardless of ship type
    let routes_top = filter_routes(ranked_routes.clone(), minimum_profit_per_time);

    match save_routes("routes_curated.json".to_string(), routes_curated.clone()) {
        Ok(_) => println!("Saved routes_curated"),
        Err(why) => println!("An error occured while saving routes_curated: {}", why),
    }
    match save_routes("routes_top.json".to_string(), routes_top.clone()) {
        Ok(_) => println!("Saved routes_top"),
        Err(why) => println!("An error occured while saving routes_top: {}", why),
    }

    let mut res = HashMap::<String, Route>::new();
    for route in routes_curated.iter() {
        res.insert(route.ship_info.model.to_string(), route.clone());
    }
    return (res, routes_top);
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

fn curate_routes(routes: Vec<Route>, ship_models: Vec<String>) -> Vec<Route> {
    let mut routes_curated = Vec::<Route>::new();
    for ship_model in ship_models {
        let ship_routes = routes
            .iter()
            .filter(|route| route.ship_info.model == ship_model)
            .cloned()
            .collect::<Vec<Route>>();
        routes_curated.push(ship_routes[0].clone());
    }
    routes_curated.sort_by(|a, b| {
        b.financials
            .credits_per_time
            .cmp(&a.financials.credits_per_time)
    });
    return routes_curated;
}

fn filter_routes(routes: Vec<Route>, minimum_profit_per_time: i32) -> Vec<Route> {
    let filtered_routes = routes
        .iter()
        .filter(|route| route.financials.credits_per_time > minimum_profit_per_time)
        .cloned()
        .collect::<Vec<Route>>();
    return filtered_routes;
}

fn save_routes(filename: String, routes: Vec<Route>) -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(filename)?;
    // write to file with serde
    serde_json::to_writer_pretty(f, &routes)?;

    Ok(())
}