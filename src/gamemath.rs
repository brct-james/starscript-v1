pub fn distance_from_coords(x1: &i32, y1: &i32, x2: &i32, y2: &i32) -> f64 {
    f64::from((x2 - x1).pow(2) + (y2 - y1).pow(2)).sqrt()
}

pub fn calculate_flight_time(scale: &f64, speed: &f64, distance: &f64) -> f64 {
    let docking_time = 30f64;
    ((distance * scale) / speed) + docking_time
}

pub fn calculate_fuel_cost(distance: &f64, ship_type: &String, target_type: &String) -> i32 {
    let mut fuel: i32;
    if ship_type == "HM-MK-III" {
        fuel = ((distance / 10f64).round() as i32) + 1;
    } else {
        fuel = ((distance / 7.5f64).round() as i32) + 1;
    }
    if target_type == "Planet" {
        match ship_type.as_str() {
            "HM-MK-III" => fuel += 1,
            "GR-MK-II" => fuel += 3,
            "GR-MK-III" => fuel += 4,
            _ => fuel += 2,
        }
    }
    //TODO: actual math for this one...
    if ship_type == "TD-MK-I" {
        fuel += 10;
    }
    return fuel;
}

// pub fn fuel_from_api_locations(distance: &f64, ship_type: &String, target_type: &String) -> i32 {
//     let mut fuel: i32;
//     if ship_type == "HM-MK-III" {
//         fuel = ((distance / 10f64).round() as i32) + 1;
//     } else {
//         fuel = ((distance / 7.5f64).round() as i32) + 1;
//     }
//     if target_type == "Planet" {
//         match ship_type.as_str() {
//             "MK-MK-III" => fuel += 1,
//             "GR-MK-II" => fuel += 3,
//             "GR-MK-III" => fuel += 4,
//             _ => fuel += 2,
//         }
//     }
//     return fuel;
// }
