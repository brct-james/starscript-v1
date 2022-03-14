use spacetraders::client::Client;
use std::time;

use super::super::shared::StarShip;
use super::super::shipmanager::ShipStatus;
use super::super::wayfinding::{generate_way_from_ship_to_way_start, Leg, StarAtlas, Way};

#[derive(
    strum_macros::Display,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    Clone,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Hash,
)]
pub enum StepSymbol {
    BuyGoods,
    SellGoods,
    TravelStart,
    TravelEnd,
    Done,
}

#[derive(Clone)]
pub struct Step {
    pub step_symbol: StepSymbol,
    pub step_name: String,
    pub next_step_symbol: StepSymbol,
}

impl Step {
    pub fn new(step_symbol: StepSymbol, next_step_symbol: StepSymbol) -> Step {
        return Step {
            step_name: step_symbol.to_string(),
            step_symbol: step_symbol,
            next_step_symbol: next_step_symbol,
        };
    }
    pub async fn run(
        &self,
        client: &Client,
        ship_status: &ShipStatus,
        staratlas: &StarAtlas,
    ) -> Result<time::Duration, Box<dyn std::error::Error>> {
        match self.step_symbol {
            StepSymbol::BuyGoods => {
                return buy_goods(client, ship_status, staratlas).await;
            }
            StepSymbol::SellGoods => {
                return sell_goods(client, ship_status, staratlas).await;
            }
            StepSymbol::TravelStart => {
                return travel_start(client, ship_status, staratlas).await;
            }
            StepSymbol::TravelEnd => {
                return travel_end(client, ship_status, staratlas).await;
            }
            _ => panic!(
                "Nonsensical StepSymbol received for step::run(): {}",
                self.step_symbol
            ),
        }
    }
}

async fn buy_fuel(client: &Client, ship: &StarShip, leg: &Leg) {
    let fuel_needed = leg.fuel_cost_to_end;
    if fuel_needed == 0i32 {
        println!("BuyFuel - have enough fuel");
        return;
    }
    let filtered_cargo = ship
        .cargo
        .as_ref()
        .unwrap()
        .iter()
        .filter(|g| g.good == spacetraders::shared::Good::Fuel)
        .collect::<Vec<&spacetraders::shared::Cargo>>();
    let owned_fuel: i32;
    if filtered_cargo.len() > 0 {
        owned_fuel = filtered_cargo[0].quantity;
        if owned_fuel >= fuel_needed {
            println!("BuyFuel - have enough fuel");
            return;
        }
    } else {
        owned_fuel = 0i32;
    }
    let units_to_buy = fuel_needed - owned_fuel;
    let multiples_of_loading_speed =
        (units_to_buy as f64 / (ship.loading_speed - 1) as f64).floor() as i32;
    let remainder = (units_to_buy % (ship.loading_speed - 1)) as i32;
    let mut orders: Vec<i32> = vec![ship.loading_speed - 1; multiples_of_loading_speed as usize];
    orders.push(remainder);
    println!("{:#?}", orders);
    for order in orders {
        if order == 0i32 {
            println!("BuyFuel");
            return;
        }
        match client
            .create_purchase_order(
                ship.id.as_ref().unwrap().to_string(),
                spacetraders::shared::Good::Fuel,
                order,
            )
            .await
        {
            Ok(_purchase_order) => {
                // println!("PO Opened: {:#?}", purchase_order);
                tokio::time::sleep(time::Duration::from_millis(500)).await; // Wait half second to avoid ratelimit
            }
            Err(why) => match why {
                spacetraders::errors::SpaceTradersClientError::ApiError(e) => {
                    if e.error.code == 400i32 {
                        // In Transit - TODO: Handle better
                        panic!("Ship In Transit");
                    } else {
                        panic!("ApiError while buying goods: {}", e);
                    }
                }
                _ => panic!("Error while buying goods: {}", why),
            },
        };
    }

    println!("BuyFuel");
    return;
}

async fn buy_goods(
    client: &Client,
    ship_status: &ShipStatus,
    staratlas: &StarAtlas,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let _ = staratlas;
    let cargo_room = ship_status.ship.space_available.unwrap();
    let apigood =
        spacetraders::shared::Good::from(ship_status.route.as_ref().unwrap().good.to_string());
    let units_to_buy = cargo_room / apigood.get_volume();
    if units_to_buy == 0 {
        println!("BuyGoods - full cargo bay, skipping purchase");
        return Ok(time::Duration::from_secs(1));
    }
    let multiples_of_loading_speed =
        (units_to_buy as f64 / (ship_status.ship.loading_speed - 1) as f64).floor() as i32;
    let remainder = (units_to_buy % (ship_status.ship.loading_speed - 1)) as i32;
    let mut orders: Vec<i32> =
        vec![ship_status.ship.loading_speed - 1; multiples_of_loading_speed as usize];
    orders.push(remainder);
    println!("{:#?}", orders);
    for order in orders {
        if order == 0i32 {
            println!("BuyGoods");
            return Ok(time::Duration::from_secs(1));
        }
        match client
            .create_purchase_order(
                ship_status.ship.id.as_ref().unwrap().to_string(),
                apigood,
                order,
            )
            .await
        {
            Ok(_purchase_order) => {
                // println!("PO Opened: {:#?}", purchase_order);
                tokio::time::sleep(time::Duration::from_millis(500)).await; // Wait half second to avoid ratelimit
            }
            Err(why) => match why {
                spacetraders::errors::SpaceTradersClientError::ApiError(e) => {
                    if e.error.code == 400i32 {
                        // In Transit - TODO: Handle better
                        panic!("Ship In Transit");
                    } else {
                        panic!("ApiError while buying goods: {}", e);
                    }
                }
                _ => panic!("Error while buying goods: {}", why),
            },
        };
    }
    println!("BuyGoods");
    return Ok(time::Duration::from_secs(1));
}

async fn sell_goods(
    client: &Client,
    ship_status: &ShipStatus,
    staratlas: &StarAtlas,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let _ = staratlas;
    let way = &ship_status.route.as_ref().unwrap().way;
    let apigood =
        spacetraders::shared::Good::from(ship_status.route.as_ref().unwrap().good.to_string());
    let mut units_to_sell = ship_status
        .ship
        .cargo
        .as_ref()
        .unwrap()
        .iter()
        .filter(|g| g.good == apigood)
        .collect::<Vec<&spacetraders::shared::Cargo>>()[0]
        .quantity;
    println!("UNITS TO SELL {:#?}", units_to_sell);
    if ship_status.ship.model == "TD-MK-I" {
        units_to_sell -= way.total_fuel_cost_to_end + 1;
    }
    println!("UNITS TO SELL {:#?}", units_to_sell);
    if units_to_sell == 0 {
        println!("SellGoods - no units of specified good in cargo bay, skipping sale");
        return Ok(time::Duration::from_secs(1));
    }
    let multiples_of_loading_speed =
        (units_to_sell as f64 / (ship_status.ship.loading_speed - 1) as f64).floor() as i32;
    let remainder = (units_to_sell % (ship_status.ship.loading_speed - 1)) as i32;
    let mut orders: Vec<i32> =
        vec![ship_status.ship.loading_speed - 1; multiples_of_loading_speed as usize];
    orders.push(remainder);
    println!("{:#?}", orders);
    for order in orders {
        if order == 0i32 {
            println!("SellGoods");
            return Ok(time::Duration::from_secs(1));
        }
        match client
            .create_sell_order(
                ship_status.ship.id.as_ref().unwrap().to_string(),
                apigood,
                order,
            )
            .await
        {
            Ok(sale_order) => {
                println!("SO Opened: {:#?}", sale_order);
                tokio::time::sleep(time::Duration::from_millis(500)).await; // Wait half second to avoid ratelimit
            }
            Err(why) => match why {
                spacetraders::errors::SpaceTradersClientError::ApiError(e) => {
                    if e.error.code == 400i32 {
                        // In Transit - TODO: Handle better
                        panic!("Ship In Transit");
                    } else {
                        panic!("ApiError while selling goods: {}", e);
                    }
                }
                _ => panic!("Error while selling goods: {}", why),
            },
        };
    }
    println!("SellGoods");
    return Ok(time::Duration::from_secs(1));
}

async fn travel(
    client: &Client,
    ship: &StarShip,
    way: &mut Way,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let mut delay_time: u64 = 0;
    loop {
        let leg = way.legs[way.current_leg_index].clone();
        println!("Travel: Leg: {} -> {}", leg.start_symbol, leg.end_symbol);
        buy_fuel(client, ship, &leg).await;
        if leg.start_symbol == leg.end_symbol {
            // At start already, skip
            println!("Travel Skipped - Already at destination");
            // Increment Leg Index
            way.incr_leg();
            // Break if executed last leg
            if way.current_leg_index == way.legs.len() {
                break;
            }
            continue;
        }
        delay_time = leg.flight_time.clone() as u64 + 1u64;
        if leg.is_warp {
            println!("{:#?} Leg is warp, attempting!", ship.id);
            match client
                .attempt_warp_jump(ship.id.as_ref().unwrap().to_string())
                .await
            {
                Ok(flight_plan) => {
                    println!("WARP FP Opened: {:#?}", flight_plan);
                    // Increment Leg Index
                    way.incr_leg();
                    // Sleep till FP should be done
                    tokio::time::sleep(time::Duration::from_secs(delay_time)).await;
                }
                Err(why) => match why {
                    spacetraders::errors::SpaceTradersClientError::ApiError(e) => {
                        match e.error.code {
                            3003i32 => {
                                // Same dest as dept - should have handled this case already...
                                println!("Same dest as dept: {}", e.error.code);
                            }
                            3002i32 => {
                                // In transit on existing flight plan
                                println!("In-Transit - Retrying Next Loop")
                            }
                            422i32 => {
                                // In transit
                                println!("In-Transit: Cant Warp - Retrying Next Loop")
                            }
                            _ => panic!("ApiError while traveling start: {}", e),
                        }
                    }
                    _ => panic!("Error while traveling start: {}", why),
                },
            };
        } else {
            println!("{:#?} Leg is flight, attempting!", ship.id);
            match client
                .create_flight_plan(
                    ship.id.as_ref().unwrap().to_string(),
                    leg.end_symbol.to_string(),
                )
                .await
            {
                Ok(flight_plan) => {
                    println!("FP Opened: {:#?}", flight_plan);
                    // Increment Leg Index
                    way.incr_leg();
                    // Sleep till FP should be done
                    tokio::time::sleep(time::Duration::from_secs(delay_time)).await;
                }
                Err(why) => match why {
                    spacetraders::errors::SpaceTradersClientError::ApiError(e) => {
                        match e.error.code {
                            3003i32 => {
                                // Same dest as dept - should have handled this case already...
                                println!("Same dest as dept: {}", e.error.code);
                            }
                            3002i32 => {
                                // In transit on existing flight plan
                                println!("In-Transit - Retrying Next Loop")
                            }
                            422i32 => {
                                // In transit
                                println!("In-Transit: Cant Warp - Retrying Next Loop")
                            }
                            _ => panic!("ApiError while traveling start: {}", e),
                        }
                    }
                    _ => panic!("Error while traveling start: {}", why),
                },
            };
        }
        // Break if executed last leg
        if way.current_leg_index == way.legs.len() {
            break;
        }
    }
    return Ok(time::Duration::from_secs(delay_time));
}

async fn travel_start(
    client: &Client,
    ship_status: &ShipStatus,
    staratlas: &StarAtlas,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let way = &ship_status.route.as_ref().unwrap().way;
    let way_to_start = generate_way_from_ship_to_way_start(&way, &ship_status.ship, &staratlas);
    println!(
        "Travel Start: {} -> {}",
        way_to_start.start_symbol, way_to_start.end_symbol
    );

    let delay_duration: time::Duration;
    match travel(client, &ship_status.ship, &mut way_to_start.clone()).await {
        Ok(dt) => delay_duration = dt,
        Err(why) => panic!("Err in travel: {}", why),
    };
    println!("TravelStart");
    return Ok(delay_duration);
}

async fn travel_end(
    client: &Client,
    ship_status: &ShipStatus,
    staratlas: &StarAtlas,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let _ = staratlas;
    let way = &ship_status.route.as_ref().unwrap().way;
    println!("Travel End: {} -> {}", way.start_symbol, way.end_symbol);
    let delay_duration: time::Duration;
    match travel(client, &ship_status.ship, &mut way.clone()).await {
        Ok(dt) => delay_duration = dt,
        Err(why) => panic!("Err in travel: {}", why),
    };
    println!("TravelEnd");
    return Ok(delay_duration);
}

pub fn generate_steps(step_list: Vec<StepSymbol>) -> Vec<Step> {
    let mut steps = Vec::<Step>::new();
    for (i, step) in step_list.iter().enumerate() {
        let next_step;
        if i + 1 == step_list.len() {
            next_step = StepSymbol::Done;
        } else {
            next_step = step_list[i + 1].clone();
        }
        match step {
            StepSymbol::BuyGoods => steps.push(Step::new(StepSymbol::BuyGoods, next_step)),
            StepSymbol::SellGoods => steps.push(Step::new(StepSymbol::SellGoods, next_step)),
            StepSymbol::TravelStart => steps.push(Step::new(StepSymbol::TravelStart, next_step)),
            StepSymbol::TravelEnd => steps.push(Step::new(StepSymbol::TravelEnd, next_step)),
            _ => panic!(
                "Attempted to generate nonsensical step: {}",
                step.to_string()
            ), //ERROR
        };
    }
    return steps;
}
