# Starscript

## TODO

- Filter routes by most profitable per ship type
- Then check that route is in-fact profitable after fuel costs
- Then start ships on that route

## Notes
Arto revealed the actual fuel calculation:

```
public static calculateFuelCost = (distance: number, fuelEfficieny: number) => {
    return Math.round((distance * fuelEfficieny) / 30) // 100 units, 4 speed -> 25 fuel
  }
const distance = FlightPlan.calculateDistance(ship.location.x, ship.location.y, planet.x, planet.y)
      const hasUndockingCost = ship.location.type === PlanetType.PLANET
      const dockingCost = (hasUndockingCost ? ship.type.dockingEfficiency : 0) + 1
      const fuelCost = FlightPlan.calculateFuelCost(distance, ship.type.fuelEfficiency) + dockingCost
```
