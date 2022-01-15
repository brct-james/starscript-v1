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