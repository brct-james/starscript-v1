use std::collections::HashMap;

#[derive(strum_macros::Display)]
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
    pub step_name: String,
    pub next_step_name: String,
    pub run: fn(options: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>>,
}

impl Step {
    pub fn new(
        step_symbol: StepSymbol,
        next_step_symbol: StepSymbol,
        run_func: fn(options: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>>,
    ) -> Step {
        return Step {
            step_name: step_symbol.to_string(),
            next_step_name: next_step_symbol.to_string(),
            run: run_func,
        };
    }
}

fn buy_fuel(options: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let _options = options;
    return Ok(());
}
fn buy_goods(options: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let _options = options;
    return Ok(());
}
fn sell_goods(options: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let _options = options;
    return Ok(());
}
fn travel_start(options: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let _options = options;
    return Ok(());
}
fn travel_end(options: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let _options = options;
    return Ok(());
}
fn finish_route(options: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let _options = options;
    return Ok(());
}

pub fn generate_steps(step_list: Vec<StepSymbol>) -> Vec<Step> {
    let mut steps = Vec::<Step>::new();
    for step in step_list {
        match step {
            StepSymbol::BuyFuel => steps.push(Step::new(
                StepSymbol::BuyFuel,
                StepSymbol::BuyGoods,
                buy_fuel,
            )),
            StepSymbol::BuyGoods => steps.push(Step::new(
                StepSymbol::BuyGoods,
                StepSymbol::SellGoods,
                buy_goods,
            )),
            StepSymbol::SellGoods => steps.push(Step::new(
                StepSymbol::SellGoods,
                StepSymbol::TravelStart,
                sell_goods,
            )),
            StepSymbol::TravelStart => steps.push(Step::new(
                StepSymbol::TravelStart,
                StepSymbol::TravelEnd,
                travel_start,
            )),
            StepSymbol::TravelEnd => steps.push(Step::new(
                StepSymbol::TravelEnd,
                StepSymbol::FinishRoute,
                travel_end,
            )),
            StepSymbol::FinishRoute => steps.push(Step::new(
                StepSymbol::FinishRoute,
                StepSymbol::Done,
                finish_route,
            )),
            _ => panic!(
                "Attempted to generate nonsensical step: {}",
                step.to_string()
            ), //ERROR
        }
    }
    return steps;
}
