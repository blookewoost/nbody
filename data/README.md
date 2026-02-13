# Three-Body Simulator - Data Files

This directory contains initial condition configuration files and simulation output.

## Configuration Files (INI format)

### `ic.ini`
Classic three-body chaotic system. Demonstrates chaotic behavior of equal-mass bodies.

### `earth_moon.ini`
Realistic Earth-Moon system with accurate masses and orbital parameters.
- Earth mass: 5.972e24 kg
- Moon mass: 7.342e22 kg
- Orbital distance: 3.844e8 m (~384,400 km)
- Moon orbital velocity: 1022 m/s

### `binary_stars.ini`
Two equal-mass stars orbiting their common center of mass.
- Both stars: 2e30 kg (about 1 solar mass each)
- Separation: 1e11 m
- Orbital velocities: ±30,000 m/s

### `sun_jupiter_saturn.ini`
Simplified solar system with three massive bodies.
- Sun: 1.989e30 kg
- Jupiter: 1.898e27 kg at 7.78e11 m with orbital velocity 13,070 m/s
- Saturn: 5.683e26 kg at 1.43e12 m with orbital velocity 9,680 m/s

## Running Simulations

To run a simulation:
```bash
./target/release/threebody-sim <config_file> <output_file>
```

Examples:
```bash
./target/release/threebody-sim ./data/ic.ini ./data/results.csv
./target/release/threebody-sim ./data/earth_moon.ini ./data/results_earth_moon.csv
./target/release/threebody-sim ./data/binary_stars.ini ./data/results_binary.csv
```

## Output Files (CSV format)

Each simulation generates a CSV file with the following format:
```
time,body0_x,body0_y,body0_z,body1_x,body1_y,body1_z,...
```

Where:
- `time`: Simulation time in seconds
- `bodyN_x`, `bodyN_y`, `bodyN_z`: Position of body N in meters

These files can be imported into visualization tools like Python/Matplotlib or other plotting software.

## Notes

- Default time step: 86,400 seconds (1 day)
- Default number of steps: 1,000
- All distances are in meters
- All masses are in kilograms
- Velocities are in meters per second
- The gravitational constant G = 6.67430e-11 m³ kg⁻¹ s⁻²
