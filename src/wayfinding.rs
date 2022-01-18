use super::gamemath;
use super::shared::StarShip;
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
    pub end_symbol: String,
    pub total_distance: i32,
    pub total_flight_time: i32,
    pub total_fuel_cost_to_end: i32,
    pub current_leg_index: usize,
    pub legs: Vec<Leg>,
}

impl Way {
    pub fn new(
        start_symbol: &String,
        end_symbol: &String,
        total_distance: &i32,
        total_flight_time: &i32,
        total_fuel_cost_to_end: &i32,
        legs: &Vec<Leg>,
    ) -> Way {
        Way {
            start_symbol: start_symbol.to_string(),
            end_symbol: end_symbol.to_string(),
            total_distance: *total_distance,
            total_flight_time: *total_flight_time,
            total_fuel_cost_to_end: *total_fuel_cost_to_end,
            current_leg_index: 0usize,
            legs: legs.clone(),
        }
    }

    pub fn incr_leg(&mut self) {
        self.current_leg_index += 1;
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Leg {
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
    pub is_warp: bool,
}

impl Leg {
    pub fn new(
        start_symbol: &String,
        end_symbol: &String,
        start_type: &String,
        start_x: &i32,
        start_y: &i32,
        end_type: &String,
        end_x: &i32,
        end_y: &i32,
        distance: &f64,
        flight_time: &f64,
        fuel_cost_to_end: &i32,
        is_warp: &bool,
    ) -> Leg {
        Leg {
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
            fuel_cost_to_end: *fuel_cost_to_end,
            is_warp: *is_warp,
        }
    }
}

pub fn generate_leg_from_symbols(
    leg_dept_symbol: &String,
    leg_dest_symbol: &String,
    ship: &StarShip,
    staratlas: &StarAtlas,
) -> Leg {
    let start_system_symbol = leg_dept_symbol.split("-").collect::<Vec<&str>>()[0].to_string();
    let end_system_symbol = leg_dest_symbol.split("-").collect::<Vec<&str>>()[0].to_string();
    let start_system = staratlas.systems[&start_system_symbol].clone();
    let start_loc = start_system.locations[&leg_dept_symbol.to_string()].clone();
    let end_system = staratlas.systems[&end_system_symbol].clone();
    let end_loc = end_system.locations[&leg_dest_symbol.to_string()].clone();
    let start_symbol = leg_dept_symbol.to_string();
    let start_type = start_loc.location_type.to_string();
    let end_symbol = leg_dest_symbol.to_string();
    let end_type = start_loc.location_type.to_string();
    let start_x = start_loc.x;
    let start_y = start_loc.y;
    let end_x = end_loc.x;
    let end_y = end_loc.y;
    let ship_speed = ship.speed;
    let ship_model = ship.model.to_string();
    let distance: f64;
    let flight_time: f64;
    let fuel_cost_to_end: i32;
    let is_warp: bool;
    if start_system_symbol == end_system_symbol {
        distance = gamemath::distance_from_coords(&start_x, &start_y, &end_x, &end_y);
        flight_time = gamemath::calculate_flight_time(&3f64, &(ship_speed as f64), &distance);
        fuel_cost_to_end = gamemath::calculate_fuel_cost(&distance, &ship_model, &end_type);
        is_warp = false;
    } else {
        // Warp
        distance = 0f64;
        flight_time = 190f64;
        fuel_cost_to_end = 0i32;
        is_warp = true;
    }
    return Leg::new(
        &start_symbol,
        &end_symbol,
        &start_type,
        &start_x,
        &start_y,
        &end_type,
        &end_x,
        &end_y,
        &distance,
        &flight_time,
        &fuel_cost_to_end,
        &is_warp,
    );
}

pub fn generate_way_from_symbols(
    start_symbol: &String,
    end_symbol: &String,
    ship: &StarShip,
    staratlas: &StarAtlas,
) -> Way {
    let mut legs = Vec::<Leg>::new();
    let mut total_distance: f64 = 0f64;
    let mut total_flight_time: f64 = 0f64;
    let mut total_fuel_cost_to_end: i32 = 0i32;
    let temp_system_links = vec![
        "OE".to_string(),
        "XV".to_string(),
        "ZY1".to_string(),
        "NA7".to_string(),
    ];
    let start_symbol_components = start_symbol.split("-").collect::<Vec<&str>>();
    let end_symbol_components = end_symbol.split("-").collect::<Vec<&str>>();
    let mut leg_dept_symbol = start_symbol.to_string();
    let mut leg_dest_symbol: String;
    let mut system_link_dept_index = temp_system_links
        .iter()
        .position(|s| {
            s.to_string() == start_symbol.split("-").collect::<Vec<&str>>()[0].to_string()
        })
        .unwrap() as i32;
    let system_link_end_index = temp_system_links
        .iter()
        .position(|s| s.to_string() == end_symbol.split("-").collect::<Vec<&str>>()[0].to_string())
        .unwrap() as i32;
    let travel_dir: i32 = if system_link_dept_index > system_link_end_index {
        -1
    } else if system_link_dept_index < system_link_end_index {
        1
    } else {
        0
    };
    if &start_symbol_components[0].to_string() == &end_symbol_components[0].to_string() {
        // In-system travel, only one leg
        leg_dest_symbol = end_symbol.to_string();
    } else {
        leg_dest_symbol = format!(
            "{}-W-{}",
            temp_system_links[system_link_dept_index as usize],
            temp_system_links[(system_link_dept_index + travel_dir) as usize]
        );
    }
    // CHECK IF DONE ELSE LOOP TILL DONE
    let mut loop_index = 0usize;
    loop {
        // Appears to be some inefficiency here - tries traveling OE-UC to OE-UC on route OE-UC -> OE-UC-AD for some reason... Make sure tests catch this, then fix
        println!(
            "leg_dest_symbol: {:#?}, start_symbol: {:#?}, ==?: {:#?}",
            leg_dest_symbol,
            start_symbol.to_string(),
            leg_dest_symbol == start_symbol.to_string()
        );
        if loop_index == 0 && leg_dest_symbol == start_symbol.to_string() {
            // This Loop's Dest is Current Location (Ship at Route Start) - Increment Dest and Skip Loop
            leg_dept_symbol = leg_dest_symbol;
            leg_dest_symbol = format!(
                "{}-W-{}",
                temp_system_links[(system_link_dept_index + travel_dir) as usize],
                temp_system_links[system_link_dept_index as usize]
            );
            if travel_dir == 0 {
                // If dest is actually in this system, go to end instead
                leg_dest_symbol = end_symbol.to_string();
            } else {
                // Increment System Link Departure Index by Travel Direction
                system_link_dept_index += travel_dir;
            }
            println!(
                "{:#?} {:#?} {:#?} {:#?}",
                travel_dir, leg_dept_symbol, leg_dest_symbol, system_link_dept_index
            );
            // Increment Loop Index
            loop_index += 1;
            continue;
        }

        // Generate and Push This Loop's Leg
        let leg = generate_leg_from_symbols(&leg_dept_symbol, &leg_dest_symbol, ship, staratlas);
        total_distance += leg.distance as f64;
        total_flight_time += leg.flight_time as f64;
        total_fuel_cost_to_end += leg.fuel_cost_to_end;
        legs.push(leg);
        if leg_dest_symbol == end_symbol.to_string() {
            // This Loop's Dest is End - No More Legs Needed - Break Loop
            break;
        }
        // Debugging index out of bounds: the len is 4 but the index is 4 at 'temp_system_links[(system_link_dept_index + travel_dir) as usize]'
        // Route:  OE-UC -> OE-UC-AD Leg: OE-UC -> OC-UC-AD
        // Location: OE-UC
        println!(
            "{:#?} dept vs dest {:#?}: dept symb: {:#?}, dest symb: {:#?}",
            system_link_dept_index, system_link_end_index, leg_dept_symbol, leg_dest_symbol
        );
        // Set Up Next Loop
        if system_link_dept_index == system_link_end_index {
            // Next loop Dest is End - Set Up Last Leg
            // TODO: CHECK IF TO MAKE SURE THIS IS ACTUALLY THE CASE
            leg_dept_symbol = leg_dest_symbol;
            leg_dest_symbol = end_symbol.to_string();
        } else {
            // Next Loop Is Intermediary Leg - Set Up Next Leg
            if let 0 = loop_index % 2 {
                // Even - {Next}-W-{Current}
                leg_dept_symbol = leg_dest_symbol;
                leg_dest_symbol = format!(
                    "{}-W-{}",
                    temp_system_links[(system_link_dept_index + travel_dir) as usize],
                    temp_system_links[system_link_dept_index as usize]
                );
                // Increment System Link Departure Index by Travel Direction
                system_link_dept_index += travel_dir;
            } else {
                // Odd - {Current}-W-{Next}
                leg_dept_symbol = leg_dest_symbol;
                leg_dest_symbol = format!(
                    "{}-W-{}",
                    temp_system_links[system_link_dept_index as usize],
                    temp_system_links[(system_link_dept_index + travel_dir) as usize]
                );
            }
        }
        // Increment Loop Index
        loop_index += 1;
    }

    return Way::new(
        &start_symbol,
        &end_symbol,
        &(total_distance.round() as i32),
        &(total_flight_time.round() as i32),
        &total_fuel_cost_to_end,
        &legs,
    );
}

pub fn generate_way_from_ship_to_way_start(
    way: &Way,
    ship: &StarShip,
    staratlas: &StarAtlas,
) -> Way {
    return generate_way_from_symbols(
        ship.location.as_ref().unwrap(),
        &way.start_symbol,
        ship,
        staratlas,
    );
}
