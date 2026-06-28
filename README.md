# Servitor — Rust MAVLink Mission Companion

A companion computer service written in Rust that connects to ArduPilot over MAVLink, parses live telemetry, and will eventually plan and execute autonomous missions in GUIDED mode. Named after the cybernetic worker-drones from Warhammer 40k.

This is a portfolio project targeting DefTech/drone software roles. Built in labs — each lab adds a concrete, demonstrable capability.

---

## Architecture

```
ArduPilot SITL
     |  UDP:14550
     v
  Servitor (this service)
     |
     +-- recv loop: parses MAVLink messages into domain structs
     |
     +-- HeartbeatBus (broadcast): fans out raw heartbeat events
     |
     +-- HeartbeatStateTracker: watches for mode/arm transitions,
                                publishes current state via watch channel
```

Raw MAVLink fields are decoded at the boundary (scaled integers, radians, bitmasks) into clean domain structs. Nothing upstream ever sees `lat * 1e7` or raw bitmask flags.

---

## Running locally

### Prerequisites

- Rust (stable)
- ArduPilot SITL built natively — clone and build from [ardupilot/ardupilot](https://github.com/ardupilot/ardupilot), follow the [native build guide](https://ardupilot.org/dev/docs/building-setup-mac.html)
- MAVProxy: `pip3 install mavproxy future gnureadline pymavlink lxml pygame`

### 1. Start SITL

From your ArduPilot repo root:

```bash
./Tools/autotest/sim_vehicle.py -v ArduCopter --speedup 1 --out udp:127.0.0.1:14550
```

> `--speedup 1` is required on Apple Silicon — higher values crash the simulator.

This starts ArduCopter SITL and opens a MAVProxy console. SITL broadcasts MAVLink over UDP to `127.0.0.1:14550`.

### 2. Start Servitor

```bash
cargo run
```

Servitor listens on `udpin:0.0.0.0:14550` (configured in `config.toml`). Logs go to stdout and `logs/servitor.log`.

### 3. Fly manually via MAVProxy console

Once both are running, use the MAVProxy console (from the SITL terminal) to command the drone:

```
mode guided        # switch to GUIDED mode
arm throttle       # arm the motors
takeoff 10         # take off to 10 metres
```

Servitor will log state transitions as they happen:

```
INFO heartbeat state has changed mode=Guided armed=true
WARN no heartbeat received within timeout ... (if SITL is killed)
```

---

## Log output example

```
INFO  drone_servitor: listening on udpin:0.0.0.0:14550
INFO  drone_servitor: heartbeat monitor started
TRACE drone_servitor: received telemetry: Heartbeat(Heartbeat { mode: Stabilize, armed: false, system_healthy: true })
INFO  drone_servitor::telemetry::heartbeat: heartbeat state has changed mode=Guided armed=false
INFO  drone_servitor::telemetry::heartbeat: heartbeat state has changed mode=Guided armed=true
```

