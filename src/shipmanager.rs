use super::shared::StarShip;
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ShipStatus {
    pub route: Option<String>,
    pub step: Option<usize>,
    pub ship: StarShip,
}

impl ShipStatus {
    pub fn new(route: Option<String>, step: Option<usize>, ship: StarShip) -> ShipStatus {
        let ship = ShipStatus {
            route: route,
            step: step,
            ship: ship,
        };
        return ship;
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ShipState {
    Active,
    Inactive,
    Scouting,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ShipManager {
    pub active: HashMap<String, ShipStatus>,
    pub inactive: HashMap<String, ShipStatus>,
    pub scouting: HashMap<String, ShipStatus>,
}

impl ShipManager {
    pub fn new() -> ShipManager {
        return ShipManager {
            active: HashMap::<String, ShipStatus>::new(),
            inactive: HashMap::<String, ShipStatus>::new(),
            scouting: HashMap::<String, ShipStatus>::new(),
        };
    }
    // pub fn get_ships(&self, subset: Option<ShipState>) -> HashMap<String, ShipStatus> {
    //     match subset {
    //         Some(shipstate) => match shipstate {
    //             ShipState::Active => return self.active.clone(),
    //             ShipState::Inactive => return self.inactive.clone(),
    //         },
    //         None => {
    //             let active = self.active.clone();
    //             let mut inactive = self.inactive.clone();
    //             let _ = active.into_iter().map(|(k, v)| inactive.insert(k, v));
    //             return inactive;
    //         }
    //     }
    // }

    pub fn get_ship(&self, id: &String) -> Option<(ShipState, ShipStatus)> {
        if self.active.contains_key(id) {
            return Some((ShipState::Active, self.active[id].clone()));
        } else if self.inactive.contains_key(id) {
            return Some((ShipState::Inactive, self.inactive[id].clone()));
        } else {
            return None;
        }
    }

    pub fn add_ship(&mut self, ship_status: ShipStatus, shipstate: ShipState) -> &mut ShipManager {
        match shipstate {
            ShipState::Active => {
                self.active.insert(
                    ship_status.ship.id.as_ref().unwrap().to_string(),
                    ship_status,
                );
            }
            ShipState::Inactive => {
                self.inactive.insert(
                    ship_status.ship.id.as_ref().unwrap().to_string(),
                    ship_status,
                );
            }
            ShipState::Scouting => {
                self.scouting.insert(
                    ship_status.ship.id.as_ref().unwrap().to_string(),
                    ship_status,
                );
            }
        }
        return self;
    }

    pub fn add_new_ship_from_api(&mut self, ship: StarShip) -> &mut ShipManager {
        self.add_ship(
            ShipStatus::new(None, None, ship.clone()),
            ShipState::Inactive,
        );
        return self;
    }

    pub fn add_ship_from_api(
        &mut self,
        ship: StarShip,
        route: Option<String>,
        step: Option<usize>,
        shipstate: ShipState,
    ) -> &mut ShipManager {
        self.add_ship(ShipStatus::new(route, step, ship), shipstate);
        return self;
    }

    pub fn load_ships_from_api(&mut self, ships: Vec<StarShip>) -> &mut ShipManager {
        for ship in ships {
            self.add_new_ship_from_api(ship);
        }
        return self;
    }

    pub fn update_ships_from_api(&mut self, ships: Vec<StarShip>) -> &mut ShipManager {
        for ship in ships {
            match self.get_ship(&(ship.id.as_ref().unwrap())) {
                Some((shipstate, shipstatus)) => {
                    self.add_ship_from_api(ship, shipstatus.route, shipstatus.step, shipstate)
                }
                None => self.add_new_ship_from_api(ship),
            };
        }
        return self;
    }

    pub fn activate_ship(&mut self, id: &String) -> &mut ShipManager {
        let localid = id.to_string();
        let ship = self.inactive.remove(&localid);
        if ship.is_some() {
            self.active.insert(localid, ship.unwrap());
        }
        return self;
    }

    // pub fn deactivate_ship(&mut self, id: &String) -> &mut ShipManager {
    //     let localid = id.to_string();
    //     let ship = self.active.remove(&localid);
    //     if ship.is_some() {
    //         self.inactive.insert(localid, ship.unwrap());
    //     }
    //     return self;
    // }

    // pub fn update_ship_route(&mut self, id: &String, new_route: &String) -> &mut ShipManager {
    //     if self.inactive.contains_key(&id.to_string()) {
    //         self.inactive
    //             .entry(id.to_string())
    //             .and_modify(|e| e.route = Some(new_route.to_string()));
    //     }
    //     return self;
    // }

    pub fn update_ship_step(&mut self, id: &String, new_step: usize) -> &mut ShipManager {
        if self.active.contains_key(&id.to_string()) {
            self.active
                .entry(id.to_string())
                .and_modify(|e| e.step = Some(new_step));
        }
        return self;
    }
    pub fn incr_ship_step(&mut self, id: &String) -> &mut ShipManager {
        if self.active.contains_key(&id.to_string()) {
            self.active
                .entry(id.to_string())
                .and_modify(|e| e.step = Some(e.step.unwrap() + 1));
        }
        return self;
    }

    pub fn update_ship_route_and_step(
        &mut self,
        id: &String,
        new_route: &String,
        new_step: usize,
    ) -> &mut ShipManager {
        if self.inactive.contains_key(&id.to_string()) {
            self.inactive.entry(id.to_string()).and_modify(|e| {
                e.step = Some(new_step);
                e.route = Some(new_route.to_string());
            });
        }
        return self;
    }

    pub fn start_route(&mut self, id: &String, route_name: &String) -> &ShipManager {
        return self
            .update_ship_route_and_step(id, route_name, 0usize)
            .activate_ship(id);
    }

    // Should very rarely need to be used, implement later
    // pub fn update_active_ship_route(&mut self, id: &String, new_route: &String) -> &mut ShipManager {

    //     return self
    // }
    // pub fn update_inactive_ship_step(&mut self, id: &String, new_step: &String) -> &mut ShipManager {

    //     return self
    // }

    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let f = std::fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open("shipmanager.json")?;
        // write to file with serde
        serde_json::to_writer_pretty(f, &self)?;

        Ok(())
    }

    // pub fn load() -> Result<Todo, std::io::Error> {
    //     // open db.json
    //     let f = std::fs::OpenOptions::new()
    //         .write(true)
    //         .create(true)
    //         .read(true)
    //         .open("shipmanager.json")?;
    //     // serialize json as HashMap
    //     match serde_json::from_reader(f) {
    //         Ok(map) => Ok(Todo { map }),
    //         Err(e) if e.is_eof() => Ok(Todo {
    //             map: HashMap::new(),
    //         }),
    //         Err(e) => panic!("An error occurred: {}", e),
    //     }
    // }
}
