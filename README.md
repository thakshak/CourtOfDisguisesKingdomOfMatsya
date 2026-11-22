# Battle of Matsya: Shadows in the Court

**Live Game:** https://thakshak.github.io/CourtOfDisguisesKingdomOfMatsya/

A 5–10 player turn-based social deduction game set in King Virata's torch-lit royal Sabha, built with **Rust (WebAssembly)** and **MQTT-over-WSS / BroadcastChannel**.

## How to Play Online
1. Open the [Live Game](https://thakshak.github.io/CourtOfDisguisesKingdomOfMatsya/).
2. Click **Create New Room** to generate a 4-character room code and share the invite link (or code) with 4–9 other players, or enter a room code to join.
3. Enter your display name after joining the **Matsya Court Lobby**.
4. Once 5–10 players have joined, the host clicks **BEGIN COUNCIL**.

## Local Development & Build
- Run unit tests:
  ```bash
  cargo test --offline
  ```
- Rebuild the GitHub Pages bundle (`./docs`):
  ```bash
  ./build_pages.sh
  ```
- Preview locally:
  ```bash
  python3 -m http.server --directory docs 8080
  ```

## Architecture & Cross-Device Networking

Because GitHub Pages serves only static files, multiplayer runs **serverlessly** using a Host-authoritative **Rust WebAssembly** engine paired with **MQTT over TLS WebSockets (`wss://`)**:

1. **Outbound TLS WebSockets (`wss://broker.emqx.io:8084/mqtt`):** Every browser/device connects via `mqtt.js` over standard outbound TLS WebSockets, avoiding WebRTC NAT/STUN/TURN and cross-browser `.local` mDNS issues.
2. **Host-Authoritative Rust WASM (`src/wasm_api.rs`):** The Host's browser loads `battle_of_matsya.wasm` (`GameEngine`) and subscribes to `matsya/sabha/<ROOM_CODE>/host`.
3. **Per-Player Isolated Topics:**
   - **Guests → Host:** Guests publish `join` and game actions to `matsya/sabha/<ROOM_CODE>/host`.
   - **Host → Each Guest:** On every state update, the Host computes `wasm_view_for_player(playerId)` and publishes each player's private `PlayerView` only to `matsya/sabha/<ROOM_CODE>/to/<PLAYER_ID>`.
4. **Same-Browser Fast Path:** Tabs within the same browser also use `BroadcastChannel("matsya-sabha-<ROOM_CODE>")` for instant local delivery.

> **Note:** Because the Host's tab holds the WASM game state and uses a shared public MQTT relay, keeping the Host tab open is required for the duration of the match.
