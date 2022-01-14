use spacetraders::client::Client;
use std::{thread, time};

use super::super::wayfinding::{StarAtlas, Way};

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
    BuyFuel,
    BuyGoods,
    SellGoods,
    TravelStart,
    TravelEnd,
    FinishRoute,
    Done,
}

pub struct Step {
    pub step_symbol: StepSymbol,
    pub step_name: String,
    pub next_step_symbol: StepSymbol,
    pub run: fn(
        client: &Client,
        ship: &spacetraders::shared::Ship,
        staratlas: &StarAtlas,
        way: &Way,
        good: &String,
    ) -> Result<time::Duration, Box<dyn std::error::Error>>,
}

impl Step {
    pub fn new(
        step_symbol: StepSymbol,
        next_step_symbol: StepSymbol,
        run_func: fn(
            client: &Client,
            ship: &spacetraders::shared::Ship,
            staratlas: &StarAtlas,
            way: &Way,
            good: &String,
        ) -> Result<time::Duration, Box<dyn std::error::Error>>,
    ) -> Step {
        return Step {
            step_name: step_symbol.to_string(),
            step_symbol: step_symbol,
            next_step_symbol: next_step_symbol,
            run: run_func,
        };
    }
}

#[tokio::main]
async fn buy_fuel(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
    good: &String,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let _ = staratlas;
    let _ = good;
    let fuel_needed = way.fuel_cost_to_end;
    let filtered_cargo = ship
        .cargo
        .iter()
        .filter(|g| g.good == spacetraders::shared::Good::Fuel)
        .collect::<Vec<&spacetraders::shared::Cargo>>();
    let owned_fuel: i32;
    if filtered_cargo.len() > 0 {
        owned_fuel = filtered_cargo[0].quantity;
        if owned_fuel > fuel_needed {
            println!("BuyFuel - have enough fuel");
            return Ok(time::Duration::from_secs(1));
        }
    }

    // let rt = tokio::runtime::Runtime::new().unwrap();
    // match rt.block_on(async {
    //     return client
    //         .create_purchase_order(
    //             ship.id.to_string(),
    //             spacetraders::shared::Good::Fuel,
    //             fuel_needed,
    //         )
    //         .await;
    // }) {
    //     Ok(purchase_order) => println!("PO Opened: {:#?}", purchase_order),
    //     Err(why) => panic!("Error while buying fuel: {}", why),
    // };
    match client
        .create_purchase_order(
            ship.id.to_string(),
            spacetraders::shared::Good::Fuel,
            fuel_needed,
        )
        .await
    {
        Ok(purchase_order) => println!("PO Opened: {:#?}", purchase_order),
        Err(why) => match why {
            spacetraders::errors::SpaceTradersClientError::ApiError(e) => {
                if e.error.code == 400i32 {
                    // In transit - wait a few seconds
                    thread::sleep(time::Duration::from_secs(5));
                    println!("In Transit - Retrying Buy Fuel in 5s");
                    return buy_fuel(client, ship, staratlas, way, good);
                } else {
                    panic!("ApiError while buying fuel: {}", e);
                }
            }
            _ => panic!("Error while buying fuel: {}", why),
        },
    };
    println!("BuyFuel");
    return Ok(time::Duration::from_secs(1));
}

#[tokio::main]
async fn buy_goods(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
    good: &String,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let _ = way;
    let _ = good;
    let cargo_room = ship.space_available;
    let apigood = spacetraders::shared::Good::from(good.to_string());
    let units_to_buy = cargo_room / apigood.get_volume();
    if units_to_buy == 0 {
        println!("BuyGoods - full cargo bay, skipping purchase");
        return Ok(time::Duration::from_secs(1));
    }
    let multiples_of_loading_speed =
        (units_to_buy as f64 / ship.loading_speed as f64).floor() as i32;
    let remainder = (units_to_buy % ship.loading_speed) as i32;
    let mut orders: Vec<i32> = vec![ship.loading_speed; multiples_of_loading_speed as usize];
    orders.push(remainder);
    println!("{:#?}", orders);
    for order in orders {
        if order == 0i32 {
            println!("BuyGoods");
            return Ok(time::Duration::from_secs(1));
        }
        match client
            .create_purchase_order(ship.id.to_string(), apigood, order)
            .await
        {
            Ok(purchase_order) => {
                println!("PO Opened: {:#?}", purchase_order);
                thread::sleep(time::Duration::from_millis(500)); // Wait half second to avoid ratelimit
            }
            Err(why) => match why {
                spacetraders::errors::SpaceTradersClientError::ApiError(e) => {
                    if e.error.code == 400i32 {
                        // In transit - wait a few seconds
                        thread::sleep(time::Duration::from_secs(5));
                        println!("In Transit - Retrying Buy Goods in 5s");
                        return buy_goods(client, ship, staratlas, way, good);
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
#[tokio::main]
async fn sell_goods(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
    good: &String,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let _ = way;
    let apigood = spacetraders::shared::Good::from(good.to_string());
    let mut units_to_sell = ship
        .cargo
        .iter()
        .filter(|g| g.good == apigood)
        .collect::<Vec<&spacetraders::shared::Cargo>>()[0]
        .quantity;
    println!("UNITS TO SELL {:#?}", units_to_sell);
    if ship.ship_type == "TD-MK-I" {
        units_to_sell -= way.fuel_cost_to_end + 1;
    }
    println!("UNITS TO SELL {:#?}", units_to_sell);
    if units_to_sell == 0 {
        println!("SellGoods - no units of specified good in cargo bay, skipping sale");
        return Ok(time::Duration::from_secs(1));
    }
    let multiples_of_loading_speed =
        (units_to_sell as f64 / ship.loading_speed as f64).floor() as i32;
    let remainder = (units_to_sell % ship.loading_speed) as i32;
    let mut orders: Vec<i32> = vec![ship.loading_speed; multiples_of_loading_speed as usize];
    orders.push(remainder);
    println!("{:#?}", orders);
    for order in orders {
        if order == 0i32 {
            println!("SellGoods");
            return Ok(time::Duration::from_secs(1));
        }
        match client
            .create_sell_order(ship.id.to_string(), apigood, order)
            .await
        {
            Ok(sale_order) => {
                println!("SO Opened: {:#?}", sale_order);
                thread::sleep(time::Duration::from_millis(500)); // Wait half second to avoid ratelimit
            }
            Err(why) => match why {
                spacetraders::errors::SpaceTradersClientError::ApiError(e) => {
                    if e.error.code == 400i32 {
                        // In transit - wait a few seconds
                        thread::sleep(time::Duration::from_secs(5));
                        println!("In Transit - Retrying Sell Goods in 5s");
                        return sell_goods(client, ship, staratlas, way, good);
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
#[tokio::main]
async fn travel_start(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
    good: &String,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let _ = good;
    println!("{} {}", way.start_symbol, way.end_symbol);
    if way.start_symbol == way.end_symbol {
        // At start already, skip
        println!("TravelStart - Already at start");
        return Ok(time::Duration::from_secs(1));
    }
    match client
        .create_flight_plan(ship.id.to_string(), way.end_symbol.to_string())
        .await
    {
        Ok(flight_plan) => println!("FP Opened: {:#?}", flight_plan),
        Err(why) => match why {
            spacetraders::errors::SpaceTradersClientError::ApiError(e) => {
                if e.error.code == 3003i32 {
                    // Same dest as dept
                    println!("e: {}", e.error.code);
                } else {
                    panic!("ApiError while traveling start: {}", e);
                }
            }
            _ => panic!("Error while traveling start: {}", why),
        },
    };
    let delay_time: u64 = way.flight_time.clone() as u64 + 1u64;
    println!("TravelStart");
    return Ok(time::Duration::from_secs(delay_time));
}

#[tokio::main]
async fn travel_end(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
    good: &String,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let _ = way;
    let _ = good;
    println!("{} {}", way.start_symbol, way.end_symbol);
    if way.start_symbol == way.end_symbol {
        // At start already, skip
        println!("TravelEnd - Already at end");
        return Ok(time::Duration::from_secs(1));
    }
    match client
        .create_flight_plan(ship.id.to_string(), way.end_symbol.to_string())
        .await
    {
        Ok(flight_plan) => println!("FP Opened: {:#?}", flight_plan),
        Err(why) => match why {
            spacetraders::errors::SpaceTradersClientError::ApiError(e) => {
                if e.error.code == 3003i32 {
                    // Same dest as dept
                    println!("e: {}", e.error.code);
                } else {
                    panic!("ApiError while traveling start: {}", e);
                }
            }
            _ => panic!("Error while traveling start: {}", why),
        },
    };
    let delay_time: u64 = way.flight_time.clone() as u64 + 1u64;
    println!("TravelEnd");
    return Ok(time::Duration::from_secs(delay_time));
}
fn finish_route(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
    good: &String,
) -> Result<time::Duration, Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let _ = way;
    let _ = good;
    println!("FinishRoute");
    return Ok(time::Duration::from_secs(1));
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
            StepSymbol::BuyFuel => steps.push(Step::new(StepSymbol::BuyFuel, next_step, buy_fuel)),
            StepSymbol::BuyGoods => {
                steps.push(Step::new(StepSymbol::BuyGoods, next_step, buy_goods))
            }
            StepSymbol::SellGoods => {
                steps.push(Step::new(StepSymbol::SellGoods, next_step, sell_goods))
            }
            StepSymbol::TravelStart => {
                steps.push(Step::new(StepSymbol::TravelStart, next_step, travel_start))
            }
            StepSymbol::TravelEnd => {
                steps.push(Step::new(StepSymbol::TravelEnd, next_step, travel_end))
            }
            StepSymbol::FinishRoute => {
                steps.push(Step::new(StepSymbol::FinishRoute, next_step, finish_route))
            }
            _ => panic!(
                "Attempted to generate nonsensical step: {}",
                step.to_string()
            ), //ERROR
        };
    }
    return steps;
}
