# Starscript

## TODO

- Filter routes by most profitable per ship type
- Then check that route is in-fact profitable after fuel costs
- Then start ships on that route

- Separate buy, sell, and travel helper functions in steps.rs
- Travel function should check if system symbols are different, if so ensure at wormhole, then warp
- Update travel step to work with Legs
- May combing buy fuel with travel step cause there will never be a buy fuel step that isnt proceeded by a travel step
- Make StarAtlas use `petgraph` crate, either in replacement or support of the hashmap (replaces temp_system_links)
- Convert test_generate_way_from_symbols into proper test suite
- Combine Travel steps
- Use returned ship info from PurchaseOrder to update shipmanager for free
- - Same for errors that tell how long till action is done
- Fix inefficiency described in wayfinding.rs comment
- - Or refactor routes so starting route for a given ship always start at ship's current location
- - - As a part of refactoring to include travel time to route start for curated routes?
- - - Curated routes should also be given for each ship in shipmanager not marked as 'scout'
- - - Oh shipmanager should have a scout/marketwatcher list

GameLoop:
- update staratlas (needs to hold market data too)
- update ship manager
- loop through ship manager:
- - any in scoutting are ignored
- - any in inactive should have routes generated, assigned, and started
- - - full routes, not routes_curated - those are for buying new ships
- - - starting a route should spawn it into another tokio runtime
- - any in active should be ignored
