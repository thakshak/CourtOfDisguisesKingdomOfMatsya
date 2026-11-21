# Battle of Matsya: Shadows in the Court

**Live Game:** https://thakshak.github.io/CourtOfDisguisesKingdomOfMatsya/

A 5–10 player turn-based social deduction game set in King Virata's torch-lit royal Sabha, built with **Rust (WebAssembly)** and **MQTT-over-WSS / BroadcastChannel**.

## How to Play Online
1. Open the [Live Game](https://thakshak.github.io/CourtOfDisguisesKingdomOfMatsya/).
2. Enter your **Google Play Games** display name.
3. Click **Create New Room** to generate a 4-character room code and share the invite link (or code) with 4–9 other players.
4. Once 5–10 players have joined the **Matsya Court Lobby**, the host clicks **BEGIN COUNCIL**.

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
