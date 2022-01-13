use spacetraders::client::Client;

use super::super::wayfinding::{StarAtlas, Way};
use std::collections::HashMap;

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
    ) -> Result<(), Box<dyn std::error::Error>>,
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
        ) -> Result<(), Box<dyn std::error::Error>>,
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
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let fuel_needed = way.fuel_cost_to_end;
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
        Err(why) => panic!("Error while buying fuel: {}", why),
    };
    return Ok(());
}
fn buy_goods(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let _ = way;
    println!("BuyGoods");
    return Ok(());
}
fn sell_goods(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let _ = way;
    println!("SellGoods");
    return Ok(());
}
fn travel_start(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let _ = way;
    println!("TravelStart");
    return Ok(());
}
fn travel_end(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let _ = way;
    println!("TravelEnd");
    return Ok(());
}
fn finish_route(
    client: &Client,
    ship: &spacetraders::shared::Ship,
    staratlas: &StarAtlas,
    way: &Way,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = client;
    let _ = ship;
    let _ = staratlas;
    let _ = way;
    println!("FinishRoute");
    return Ok(());
}

pub fn generate_steps(step_list: Vec<StepSymbol>) -> HashMap<StepSymbol, Step> {
    let mut steps = HashMap::<StepSymbol, Step>::new();
    for (i, step) in step_list.iter().enumerate() {
        let next_step;
        if i + 1 == step_list.len() {
            next_step = StepSymbol::Done;
        } else {
            next_step = step_list[i + 1].clone();
        }
        match step {
            StepSymbol::BuyFuel => steps.insert(
                step.clone(),
                Step::new(StepSymbol::BuyFuel, next_step, buy_fuel),
            ),
            StepSymbol::BuyGoods => steps.insert(
                step.clone(),
                Step::new(StepSymbol::BuyGoods, next_step, buy_goods),
            ),
            StepSymbol::SellGoods => steps.insert(
                step.clone(),
                Step::new(StepSymbol::SellGoods, next_step, sell_goods),
            ),
            StepSymbol::TravelStart => steps.insert(
                step.clone(),
                Step::new(StepSymbol::TravelStart, next_step, travel_start),
            ),
            StepSymbol::TravelEnd => steps.insert(
                step.clone(),
                Step::new(StepSymbol::TravelEnd, next_step, travel_end),
            ),
            StepSymbol::FinishRoute => steps.insert(
                step.clone(),
                Step::new(StepSymbol::FinishRoute, next_step, finish_route),
            ),
            _ => panic!(
                "Attempted to generate nonsensical step: {}",
                step.to_string()
            ), //ERROR
        };
    }
    return steps;
}
