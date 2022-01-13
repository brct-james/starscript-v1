use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ShipManager {
    pub active: HashMap<String, Ship>,
    pub inactive: HashMap<String, Ship>,
}

impl ShipManager {
    pub fn new() -> ShipManager {
        return ShipManager {
            active: HashMap::<String, Ship>::new(),
            inactive: HashMap::<String, Ship>::new(),
        };
    }
    pub fn list_ships(self, subset: String) -> HashMap<String, Ship> {
        match subset.as_str() {
            "ACTIVE" => return self.active,
            "INACTIVE" => return self.inactive,
            "ALL" => {
                let active = self.active.clone();
                let mut inactive = self.inactive.clone();
                let _ = active.into_iter().map(|(k, v)| inactive.insert(k, v));
                return inactive;
            }
            _ => return HashMap::<String, Ship>::new(),
        }
    }

    pub fn add_new_ship(mut self, id: String, model: String, location: String) -> ShipManager {
        let ship = Ship {
            id: id.clone(),
            model: model,
            location: location,
            route: None,
            step: None,
        };
        self.inactive.insert(id, ship);
        return self;
    }

    pub fn activate_ship(mut self, id: &String) -> ShipManager {
        let localid = id.to_string();
        let ship = self.inactive.remove(&localid);
        if ship.is_some() {
            self.active.insert(localid, ship.unwrap());
        }
        return self;
    }

    pub fn deactivate_ship(mut self, id: &String) -> ShipManager {
        let localid = id.to_string();
        let ship = self.active.remove(&localid);
        if ship.is_some() {
            self.inactive.insert(localid, ship.unwrap());
        }
        return self;
    }

    pub fn update_ship_route(mut self, id: &String, new_route: String) -> ShipManager {
        let localid = id.to_string();
        if self.inactive.contains_key(&localid) {
            self.inactive
                .entry(localid)
                .and_modify(|e| e.route = Some(new_route));
        }
        return self;
    }

    pub fn update_ship_step(mut self, id: &String, new_step: String) -> ShipManager {
        let localid = id.to_string();
        if self.active.contains_key(&localid) {
            self.active
                .entry(localid)
                .and_modify(|e| e.step = Some(new_step));
        }
        return self;
    }

    pub fn start_route(
        self,
        id: String,
        route_name: String,
        first_step_name: String,
    ) -> ShipManager {
        return self
            .update_ship_route(&id, route_name)
            .update_ship_step(&id, first_step_name)
            .activate_ship(&id);
    }

    // Should very rarely need to be used, implement later
    // pub fn update_active_ship_route(mut self, id: String, new_route: String) -> ShipManager {

    //     return self
    // }
    // pub fn update_active_ship_step(mut self, id: String, new_step: String) -> ShipManager {

    //     return self
    // }

    // pub fn save(shipmanager: ShipManager) -> Result<(), Box<dyn std::error::Error>> {
    //     let f = std::fs::OpenOptions::new()
    //         .truncate(true)
    //         .write(true)
    //         .create(true)
    //         .open("shipmanager.json")?;
    //     // write to file with serde
    //     serde_json::to_writer_pretty(f, &shipmanager)?;

    //     Ok(())
    // }

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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Ship {
    id: String,
    model: String,
    location: String,
    route: Option<String>,
    step: Option<String>,
}
