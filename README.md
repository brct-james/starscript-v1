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

And travel time:

```
const SYSTEM_SCALE = 3
public static calculateArrival = (distance: number, speed: number) => {
    const seconds = (distance * SYSTEM_SCALE) / speed

    return moment()
      .add(seconds + 30, 'seconds')
      .toDate()
  }
```

Distance calc:
```
public static calculateDistance = (fromX: number, fromY: number, toX: number, toY: number) => {
    const from = new Flatten.Point(fromX, fromY)
    const to = new Flatten.Point(toX, toY)

    return Math.round(from.distanceTo(to)[0])
  }
```
