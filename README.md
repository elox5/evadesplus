# Evades+

This is an open-source self-hostable reimagination of [evades.io](https://evades.io) in Rust, focusing on performance and modularity.

## Why?
Evades has its backend written in Python, which makes it notoriously slow with a multitude of performance bugs and general lag spikes even on certain good network enviroments. This project aims to address those issues by having a highly performant and fault-proof backend written in Rust, and by using the new [WebTransport API](https://developer.mozilla.org/en-US/docs/Web/API/WebTransport_API) as the network protocol.

This implementation is also completely open source, and aims to be modular and self-hostable, and to allow simple modding support in the future.

## Self-hosting instructions
1. Clone the project:
`git clone https://github.com/elox5/evadesplus`

2. Build the dependencies: `cargo build`

3. Load custom map files by putting them in the `maps` directory

4. Set up environment variables

5. Start the server with `cargo run --release`

## Environment variable reference

### Network info
- `LOCAL_IP`: The IP to start the server on (default: `127.0.0.1`)
- `HTTPS_PORT`: The port to serve the game on (default: `443`)
- `HTTP_PORT`: The port for the HTTP redirect (default: `80`)

### Client setup
- `CLIENT_PATH`:  the path to the directory containing static client files (default: `client/dist`)

### Map data
- `MAP_PATH`: The path to the directory containing map files (default: `maps`)
- `MAPS`: The list of maps to load (required)
- `START_AREA_ID`: The area ID to load new players into (required)

### Game settings
- `SIMULATION_FRAMERATE`: The framerate the simulation loop runs on (default: `60`)

## Issues with WebTransport for local hosting
Recent updates to major browsers prevent WebTransport connections from being established for self-hosted SSL certificates by default. This means you need a valid CA-signed SSL certificate for the WebTransport connection to succeed. To bypass this, follow these steps:

#### Firefox:
Currently, there is no known fix for Firefox versions >= 133. Consider downgrading Firefox to version 132 or earlier.

#### Chromium-based browsers:
Head to `chrome://flags` and set `#webtransport-developer-mode` to `Enabled`

## What's next?

### Engine
- Implementing a modular Enemy Behavior system
- Implementing an ability system
- Spatial hashing for improved collision detection performance

### Server
- Proper support for hosting outside a local network
- Improved debug support, including an interactive server console and debug rendering capability

### Client
- Caching enemy information on the client to reduce network load
- Eventually, a WebAssembly-powered smart client which requires only periodical sync packets

### The future
- An account system
- A map editor
- Modding support
- Better documentation

## Known issues
- Clients on Chromium browsers don't announce disconnection to the server, leading to their heroes staying loaded for a period of time afterwards
- Clients on Chromium browsers being flooded with `STOP_SENDING` errors
- Chromium browsers not connecting to the server on `localhost` (only `127.0.0.1`)