use spacetraders::shared::{Cargo, Good, PurchaseLocation, Ship, ShipForSale};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct StarShip {
    pub id: Option<String>,
    pub location: Option<String>,
    pub cargo: Option<Vec<Cargo>>,
    pub space_available: Option<i32>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub flight_plan_id: Option<String>,
    pub model: String,
    pub class: String,
    pub max_cargo: i32,
    pub speed: i32,
    pub manufacturer: String,
    pub plating: i32,
    pub weapons: i32,
    pub purchase_locations: Option<Vec<PurchaseLocation>>,
    pub restricted_goods: Option<Vec<Good>>,
    pub loading_speed: i32,
}

impl StarShip {
    // pub fn new() -> StarShip {}
}

// Pattern for vec of StarShip:
// <<<VEC>>>
// .iter().map(|s| StarShip::from(s.clone())).collect::<Vec<StarShip>>()

impl From<ShipForSale> for StarShip {
    fn from(ship: ShipForSale) -> Self {
        StarShip {
            id: None,
            location: None,
            cargo: None,
            space_available: None,
            x: None,
            y: None,
            flight_plan_id: None,
            model: ship.ship_type,
            class: ship.class,
            max_cargo: ship.max_cargo,
            speed: ship.speed,
            manufacturer: ship.manufacturer,
            plating: ship.plating,
            weapons: ship.weapons,
            purchase_locations: Some(ship.purchase_locations),
            restricted_goods: ship.restricted_goods,
            loading_speed: ship.loading_speed,
        }
    }
}

impl From<Ship> for StarShip {
    fn from(ship: Ship) -> Self {
        StarShip {
            id: Some(ship.id),
            location: ship.location,
            cargo: Some(ship.cargo),
            space_available: Some(ship.space_available),
            x: ship.x,
            y: ship.y,
            flight_plan_id: ship.flight_plan_id,
            model: ship.ship_type,
            class: ship.class,
            max_cargo: ship.max_cargo,
            speed: ship.speed,
            manufacturer: ship.manufacturer,
            plating: ship.plating,
            weapons: ship.weapons,
            purchase_locations: None,
            restricted_goods: ship.restricted_goods,
            loading_speed: ship.loading_speed,
        }
    }
}
