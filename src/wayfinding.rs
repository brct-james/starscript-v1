use super::gamemath;
use super::traderoutes::routes::Route;
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct StarAtlas {
    pub systems: HashMap<String, StarSystem>,
}

impl StarAtlas {
    pub fn new() -> StarAtlas {
        StarAtlas {
            systems: HashMap::<String, StarSystem>::new(),
        }
    }

    pub fn add_system(
        &mut self,
        symbol: String,
        name: String,
        locations: Vec<spacetraders::shared::Location>,
    ) -> &StarAtlas {
        self.systems.insert(
            symbol.to_string(),
            StarSystem {
                symbol: symbol,
                name: name,
                locations: locations
                    .iter()
                    .map(|l| (l.symbol.to_string(), l.clone()))
                    .collect(),
            },
        );
        return self;
    }

    // pub fn add_location(&mut self) -> &StarAtlas {
    //     return self;
    // }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct StarSystem {
    symbol: String,
    name: String,
    pub locations: HashMap<String, spacetraders::shared::Location>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Way {
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

impl Way {
    pub fn new(
        start_symbol: &String,
        start_type: &String,
        start_x: &i32,
        start_y: &i32,
        end_symbol: &String,
        end_type: &String,
        end_x: &i32,
        end_y: &i32,
        ship_speed: &i32,
        ship_type: &String,
    ) -> Way {
        let distance = gamemath::distance_from_coords(start_x, start_y, end_x, end_y);
        let flight_time = gamemath::calculate_flight_time(&3f64, &(*ship_speed as f64), &distance);
        let fuel_cost_to_end = gamemath::calculate_fuel_cost(&distance, &ship_type, &end_type);
        let fuel_cost_to_start = gamemath::calculate_fuel_cost(&distance, &ship_type, &start_type);
        Way {
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
        }
    }
}

pub fn generate_way_from_symbols(
    start_symbol: &String,
    end_symbol: &String,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
) -> Way {
    let start_system = staratlas.systems[&start_symbol[..2]].clone();
    let start_loc = start_system.locations[&start_symbol.to_string()].clone();
    let end_system = staratlas.systems[&end_symbol[..2]].clone();
    let end_loc = end_system.locations[&end_symbol.to_string()].clone();
    let start_symbol = start_symbol.to_string();
    let start_type = start_loc.location_type.to_string();
    let start_x = start_loc.x;
    let start_y = start_loc.y;
    let end_symbol = end_symbol.to_string();
    let end_type = start_loc.location_type.to_string();
    let end_x = end_loc.x;
    let end_y = end_loc.y;
    let ship_speed = ship.speed;
    let ship_type = ship.ship_type.to_string();
    return Way::new(
        &start_symbol,
        &start_type,
        &start_x,
        &start_y,
        &end_symbol,
        &end_type,
        &end_x,
        &end_y,
        &ship_speed,
        &ship_type,
    );
}

pub fn generate_way_to_route_start(
    route: &Route,
    start_symbol: &String,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
) -> Way {
    return generate_way_from_symbols(
        start_symbol,
        &route.wayfinding.start_symbol,
        ship,
        staratlas,
    );
}
