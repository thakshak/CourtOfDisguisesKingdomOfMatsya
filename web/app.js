(() => {
  const params = new URLSearchParams(window.location.search);
  const rawRoom = (params.get("room") || "").toUpperCase().replace(/[^A-Z0-9]/g, "");
  let roomCode = rawRoom.length === 4 ? rawRoom : "";

  let playerId = sessionStorage.getItem("matsya_player_id");
  if (!playerId) {
    playerId = "p_" + Math.random().toString(36).slice(2, 10);
    sessionStorage.setItem("matsya_player_id", playerId);
  }

  let hasSetName = Boolean(params.get("name") || sessionStorage.getItem("matsya_custom_name"));
  let displayName =
    params.get("name") ||
    sessionStorage.getItem("matsya_custom_name") ||
    `Envoy-${playerId.slice(2, 6).toUpperCase()}`;

  let currentView = null;
  let selectedTeamIds = new Set();
  let peekingDisguise = false;
  let lastPhase = null;
  let toastTimer = null;

  // WASM + MQTT-over-WSS + BroadcastChannel state
  let wasm = null;
  let mqttClient = null;
  let bc = null;
  let isHostNode = false;
  let hostTransport = null; // "bc" | "mqtt"
  let guestJoinTimer = null;
  const guestTransports = new Map(); // playerId -> "bc" | "mqtt"

  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  const mainStage = document.getElementById("main-stage");
  const chatMessagesEl = document.getElementById("chat-messages");
  const chatForm = document.getElementById("chat-form");
  const chatInput = document.getElementById("chat-input");
  const gpgNameLabel = document.getElementById("gpg-name-label");
  const roomCodeLabel = document.getElementById("room-code-label");
  const copyLinkBtn = document.getElementById("copy-link-btn");
  const disguisePeekBtn = document.getElementById("disguise-peek-btn");
  const helpBtn = document.getElementById("help-btn");
  const helpModal = document.getElementById("help-modal");
  const closeHelpBtn = document.getElementById("close-help-btn");
  const audioToggleBtn = document.getElementById("audio-toggle-btn");
  const bgmAudio = document.getElementById("bgm-audio");
  const toastBanner = document.getElementById("toast-banner");

  roomCodeLabel.textContent = roomCode || "----";
  gpgNameLabel.textContent = hasSetName ? displayName : "Guest";
  if (roomCode) {
    copyLinkBtn.classList.remove("hidden");
  }

  function showToast(msg) {
    toastBanner.textContent = msg;
    toastBanner.classList.remove("hidden");
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toastBanner.classList.add("hidden");
    }, 4500);
  }

  // ---- WASM Bridge ----
  async function loadWasm() {
    if (wasm) return wasm;
    const response = await fetch("./battle_of_matsya.wasm");
    const bytes = await response.arrayBuffer();
    const { instance } = await WebAssembly.instantiate(bytes, {});
    wasm = instance.exports;
    return wasm;
  }

  function writeWasmStr(str) {
    const bytes = encoder.encode(str);
    const ptr = wasm.wasm_alloc(bytes.length);
    new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
    return { ptr, len: bytes.length };
  }

  function readWasmOut() {
    const ptr = wasm.wasm_out_ptr();
    const len = wasm.wasm_out_len();
    return decoder.decode(new Uint8Array(wasm.memory.buffer, ptr, len));
  }

  function wasmInitRoom(code) {
    const r = writeWasmStr(code);
    const seedArr = new Uint32Array(2);
    crypto.getRandomValues(seedArr);
    const seed = (BigInt(seedArr[0]) << 32n) | BigInt(seedArr[1]);
    wasm.wasm_init_room(r.ptr, r.len, seed);
    wasm.wasm_dealloc(r.ptr, r.len);
  }

  function wasmDispatch(pid, cmdObj) {
    const p = writeWasmStr(pid);
    const c = writeWasmStr(JSON.stringify(cmdObj));
    const rc = wasm.wasm_dispatch(p.ptr, p.len, c.ptr, c.len);
    wasm.wasm_dealloc(p.ptr, p.len);
    wasm.wasm_dealloc(c.ptr, c.len);
    const out = readWasmOut();
    return { ok: rc === 0, out };
  }

  function wasmViewForPlayer(pid) {
    const p = writeWasmStr(pid);
    const rc = wasm.wasm_view_for_player(p.ptr, p.len);
    wasm.wasm_dealloc(p.ptr, p.len);
    const out = readWasmOut();
    if (rc !== 0) return null;
    return JSON.parse(out);
  }

  function wasmPlayerIds() {
    const rc = wasm.wasm_player_ids();
    if (rc !== 0) return [];
    return JSON.parse(readWasmOut());
  }

  function topicHost(code) {
    return `matsya/sabha/${code}/host`;
  }

  function topicToPlayer(code, pid) {
    return `matsya/sabha/${code}/to/${pid}`;
  }

  function sendToGuest(pid, payload) {
    const transport = guestTransports.get(pid);
    if (transport === "bc" && bc) {
      bc.postMessage({ to: pid, ...payload });
    } else if (mqttClient && mqttClient.connected) {
      mqttClient.publish(topicToPlayer(roomCode, pid), JSON.stringify(payload), { qos: 1 });
    }
  }

  function broadcastAllViews() {
    const ids = wasmPlayerIds();
    for (const pid of ids) {
      const view = wasmViewForPlayer(pid);
      if (!view) continue;
      if (pid === playerId) {
        applyStateView(view);
      } else {
        sendToGuest(pid, { type: "state", view });
      }
    }
  }

  function handleHostCommand(senderId, cmd) {
    if (cmd.type === "send_chat") {
      cmd = { ...cmd, timestamp_ms: Date.now() };
    }
    const res = wasmDispatch(senderId, cmd);
    if (res.ok) {
      broadcastAllViews();
    } else if (senderId === playerId) {
      showToast(res.out);
    } else {
      sendToGuest(senderId, { type: "error", message: res.out });
    }
  }

  function applyStateView(view) {
    currentView = view;
    if (lastPhase !== currentView.phase) {
      selectedTeamIds.clear();
      lastPhase = currentView.phase;
    }
    render();
  }

  function sendCmd(cmd) {
    if (isHostNode) {
      handleHostCommand(playerId, cmd);
    } else if (hostTransport === "bc" && bc) {
      bc.postMessage({ from: playerId, ...cmd });
    } else if (hostTransport === "mqtt" && mqttClient && mqttClient.connected) {
      mqttClient.publish(topicHost(roomCode), JSON.stringify({ from: playerId, ...cmd }), {
        qos: 1,
      });
    } else {
      showToast("Connecting to Matsya Court host...");
    }
  }

  function generateRoomCode() {
    const chars = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    const arr = new Uint8Array(4);
    crypto.getRandomValues(arr);
    let out = "";
    for (let i = 0; i < 4; i++) {
      out += chars[arr[i] % chars.length];
    }
    return out;
  }

  const MQTT_BROKER_URL = "wss://broker.emqx.io:8084/mqtt";

  async function startAsHost(code) {
    isHostNode = true;
    roomCode = code;
    roomCodeLabel.textContent = roomCode;
    copyLinkBtn.classList.remove("hidden");
    window.history.replaceState({}, "", `${window.location.pathname}?room=${encodeURIComponent(roomCode)}`);

    mainStage.innerHTML = `<div class="card"><h2>Opening King Virata's Sabha...</h2><p>Loading Rust WASM engine &amp; connecting court relay...</p></div>`;

    await loadWasm();
    wasmInitRoom(roomCode);
    wasmDispatch(playerId, { type: "join", player_id: playerId, display_name: displayName });
    broadcastAllViews();

    // Same-browser tab fast-path
    if ("BroadcastChannel" in window) {
      bc = new BroadcastChannel(`matsya-sabha-${roomCode}`);
      bc.onmessage = (ev) => {
        const msg = ev.data;
        if (!msg || typeof msg !== "object" || msg.to) return;
        if (msg.type === "join") {
          const pid = msg.player_id;
          const res = wasmDispatch(pid, {
            type: "join",
            player_id: pid,
            display_name: msg.display_name,
          });
          if (res.ok) {
            guestTransports.set(pid, "bc");
            broadcastAllViews();
          } else {
            bc.postMessage({ to: pid, type: "error", message: res.out });
          }
        } else if (msg.from) {
          handleHostCommand(msg.from, msg);
        }
      };
    }

    // Cross-browser / cross-device MQTT over WSS
    mqttClient = mqtt.connect(MQTT_BROKER_URL, {
      clientId: `matsya_host_${roomCode}_${Math.random().toString(36).slice(2, 7)}`,
      clean: true,
      reconnectPeriod: 2500,
    });

    mqttClient.on("connect", () => {
      mqttClient.subscribe(topicHost(roomCode), { qos: 1 }, (err) => {
        if (!err) {
          showToast(`Sabha ${roomCode} is open for all browsers & devices!`);
        }
      });
    });

    mqttClient.on("message", (_topic, payload) => {
      let msg;
      try {
        msg = JSON.parse(payload.toString());
      } catch {
        return;
      }
      if (!msg || typeof msg !== "object") return;
      if (msg.type === "join") {
        const pid = msg.player_id;
        const res = wasmDispatch(pid, {
          type: "join",
          player_id: pid,
          display_name: msg.display_name,
        });
        if (res.ok) {
          guestTransports.set(pid, "mqtt");
          broadcastAllViews();
        } else {
          mqttClient.publish(
            topicToPlayer(roomCode, pid),
            JSON.stringify({ type: "error", message: res.out }),
            { qos: 1 }
          );
        }
        return;
      }
      if (msg.from) {
        handleHostCommand(msg.from, msg);
      }
    });

    mqttClient.on("error", (err) => {
      showToast(`Court relay error: ${err.message || err}`);
    });
  }

  function joinAsGuest(code) {
    isHostNode = false;
    roomCode = code;
    roomCodeLabel.textContent = roomCode;
    copyLinkBtn.classList.remove("hidden");
    window.history.replaceState({}, "", `${window.location.pathname}?room=${encodeURIComponent(roomCode)}`);

    mainStage.innerHTML = `
      <div class="card">
        <h2>Joining Sabha ${escapeHtml(roomCode)}...</h2>
        <p id="join-status-text">Connecting to host...</p>
        <div class="action-row">
          <button id="back-to-gate-btn" class="btn btn-secondary btn-sm" type="button">Back to Room Entry</button>
        </div>
      </div>
    `;
    document.getElementById("back-to-gate-btn").addEventListener("click", () => {
      clearTimeout(guestJoinTimer);
      if (mqttClient) mqttClient.end(true);
      if (bc) bc.close();
      roomCode = "";
      window.history.replaceState({}, "", window.location.pathname);
      copyLinkBtn.classList.add("hidden");
      roomCodeLabel.textContent = "----";
      renderRoomGate();
    });

    // Try instant same-browser BroadcastChannel first
    if ("BroadcastChannel" in window) {
      bc = new BroadcastChannel(`matsya-sabha-${roomCode}`);
      bc.onmessage = (ev) => {
        const msg = ev.data;
        if (!msg || msg.to !== playerId) return;
        if (msg.type === "state") {
          hostTransport = "bc";
          clearTimeout(guestJoinTimer);
          applyStateView(msg.view);
        } else if (msg.type === "error") {
          showToast(msg.message);
        }
      };
      bc.postMessage({ type: "join", player_id: playerId, display_name: displayName });
    }

    // Cross-browser / cross-device MQTT over WSS
    mqttClient = mqtt.connect(MQTT_BROKER_URL, {
      clientId: `matsya_guest_${playerId}_${Math.random().toString(36).slice(2, 7)}`,
      clean: true,
      reconnectPeriod: 2500,
    });

    let attempts = 0;
    const sendMqttJoin = () => {
      if (hostTransport === "bc" || currentView) return;
      attempts += 1;
      const statusEl = document.getElementById("join-status-text");
      if (statusEl) {
        statusEl.textContent = `Joining room ${roomCode} (attempt ${attempts})...`;
      }
      mqttClient.publish(
        topicHost(roomCode),
        JSON.stringify({ type: "join", player_id: playerId, display_name: displayName }),
        { qos: 1 }
      );
      if (attempts < 5) {
        guestJoinTimer = setTimeout(sendMqttJoin, 2500);
      } else {
        const el = document.getElementById("join-status-text");
        if (el) {
          el.textContent = `No response from host for room ${roomCode}. Make sure the host's tab is open.`;
        }
      }
    };

    mqttClient.on("connect", () => {
      if (hostTransport === "bc") return;
      mqttClient.subscribe(topicToPlayer(roomCode, playerId), { qos: 1 }, (err) => {
        if (!err) {
          sendMqttJoin();
        }
      });
    });

    mqttClient.on("message", (_topic, payload) => {
      if (hostTransport === "bc") return;
      let msg;
      try {
        msg = JSON.parse(payload.toString());
      } catch {
        return;
      }
      if (msg.type === "error") {
        showToast(msg.message);
      } else if (msg.type === "state") {
        hostTransport = "mqtt";
        clearTimeout(guestJoinTimer);
        applyStateView(msg.view);
      }
    });

    mqttClient.on("error", (err) => {
      showToast(`Network error: ${err.message || err}`);
    });
  }

  copyLinkBtn.addEventListener("click", () => {
    const base = `${window.location.origin}${window.location.pathname}`;
    const inviteUrl = `${base}?room=${encodeURIComponent(roomCode)}`;
    navigator.clipboard?.writeText(inviteUrl);
    showToast(`Invite link copied for room ${roomCode}!`);
  });

  disguisePeekBtn.addEventListener("click", () => {
    peekingDisguise = !peekingDisguise;
    render();
  });

  helpBtn.addEventListener("click", () => {
    helpModal.classList.remove("hidden");
  });

  closeHelpBtn.addEventListener("click", () => {
    helpModal.classList.add("hidden");
  });

  helpModal.addEventListener("click", (e) => {
    if (e.target === helpModal) {
      helpModal.classList.add("hidden");
    }
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      helpModal.classList.add("hidden");
    }
  });

  audioToggleBtn.addEventListener("click", () => {
    if (bgmAudio.paused) {
      bgmAudio.volume = 0.35;
      bgmAudio.play().then(() => {
        audioToggleBtn.textContent = "Music: On";
      }).catch(() => {
        showToast("Tap again to enable court music.");
      });
    } else {
      bgmAudio.pause();
      audioToggleBtn.textContent = "Music: Off";
    }
  });

  chatForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const text = chatInput.value.trim();
    if (!text) return;
    sendCmd({ type: "send_chat", text });
    chatInput.value = "";
  });

  function escapeHtml(str) {
    return String(str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function renderChat(messages) {
    const atBottom =
      chatMessagesEl.scrollHeight - chatMessagesEl.scrollTop <= chatMessagesEl.clientHeight + 40;
    chatMessagesEl.innerHTML = messages
      .map(
        (m) =>
          `<div class="chat-msg"><strong>${escapeHtml(m.sender_name)}:</strong> ${escapeHtml(
            m.text
          )}</div>`
      )
      .join("");
    if (atBottom) {
      chatMessagesEl.scrollTop = chatMessagesEl.scrollHeight;
    }
  }

  function renderBattlesTrack(view) {
    const items = view.battles
      .map((b, idx) => {
        const active = idx === view.current_battle_index && view.phase !== "victory";
        let cls = "battle-medallion";
        let icon = "./assets/images/token_compass_star.png";
        let status = `Team: ${b.team_size}`;
        if (b.outcome) {
          if (b.outcome.winner === "dharma") {
            cls += " won-dharma";
            icon = "./assets/images/icon_crown_defender.png";
            status = "Dharma Win";
          } else {
            cls += " won-infiltrators";
            icon = "./assets/images/icon_shield_rival.png";
            status = "Infiltrator Win";
          }
        } else if (active) {
          cls += " active";
        } else {
          cls += " upcoming";
        }
        const sabNote = b.sabotages_required > 1 ? ` • ${b.sabotages_required} Sabotages` : "";
        return `
          <div class="${cls}">
            ${active ? '<span class="active-battle-pill">Current Battle</span>' : ""}
            <div class="b-num">Battle ${b.number}</div>
            <img src="${icon}" alt="${status}" />
            <div class="b-name">${escapeHtml(b.name)}</div>
            <div class="b-size">${status}${sabNote}</div>
          </div>
        `;
      })
      .join("");

    const flames = [1, 2, 3, 4, 5]
      .map(
        (step) => `
        <div class="deadlock-flame ${view.deadlock_count >= step ? "lit" : ""}" title="Deadlock ${step}/5">
          <img src="./assets/images/icon_deadlock_flame.png" alt="Deadlock Flame" />
        </div>
      `
      )
      .join("");

    return `
      <div class="card">
        <div class="battles-track">${items}</div>
        <div class="deadlock-bar">
          <div>
            <strong>Deadlock Tracker:</strong> ${view.deadlock_count} / 5 consecutive rejections
            <span style="color: var(--text-muted); font-size: 0.85rem;">(5 rejections = Infiltrators win)</span>
          </div>
          <div class="deadlock-steps">${flames}</div>
        </div>
      </div>
    `;
  }

  function renderLobby(view) {
    const me = view.players.find((p) => p.id === view.self_id);
    const isHost = me?.is_host;
    const count = view.players.length;
    const canStart = count >= 5 && count <= 10;

    const banners = view.players
      .map(
        (p) => `
        <div class="sandstone-banner">
          <span class="player-name">${escapeHtml(p.display_name)}</span>
          <div style="display:flex; gap:5px; align-items:center;">
            ${p.is_host ? '<span class="tag tag-host">Host</span>' : ""}
            ${p.id === view.self_id ? '<span class="tag tag-you">You</span>' : ""}
            ${!p.connected ? '<span class="tag tag-offline">Offline</span>' : ""}
          </div>
        </div>
      `
      )
      .join("");

    mainStage.innerHTML = `
      ${
        !hasSetName
          ? `<div class="card needs-action">
              <div class="action-cue-badge">Your Action Required</div>
              <h2 class="screen-title">Welcome to Sabha ${escapeHtml(roomCode)}</h2>
              <p class="screen-desc">Enter your name so the rest of the court knows who you are:</p>
              <form id="gpg-form" class="profile-row" style="margin-bottom:0;">
                <input id="gpg-input" type="text" maxlength="24" placeholder="Enter your name..." autocomplete="off" required />
                <button type="submit" class="btn btn-gold">Save Name</button>
              </form>
            </div>`
          : ""
      }
      <div class="card ${hasSetName && isHost && canStart ? "needs-action" : ""}">
        ${hasSetName && isHost && canStart ? '<div class="action-cue-badge">Your Action Required</div>' : ""}
        <h2 class="screen-title">Matsya Court Lobby</h2>
        <p class="screen-desc">
          Gather 5 to 10 courtiers in King Virata's Sabha. Share room code <strong>${escapeHtml(
            roomCode
          )}</strong> or click <strong>Copy Invite</strong>.
        </p>

        ${
          hasSetName
            ? `<form id="gpg-form" class="profile-row">
                <label for="gpg-input" style="font-size:0.9rem;"><strong>Your Name:</strong></label>
                <input id="gpg-input" type="text" maxlength="24" value="${escapeHtml(displayName)}" />
                <button type="submit" class="btn btn-secondary btn-sm">Update Name</button>
              </form>`
            : ""
        }

        <div style="display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:8px;">
          <strong>Joined Courtiers: ${count} / 10 (Minimum 5 required)</strong>
          ${
            isHost
              ? `<button id="begin-council-btn" class="btn btn-gold" ${canStart ? "" : "disabled"}>BEGIN COUNCIL</button>`
              : `<span style="color: var(--text-muted);">Waiting for host to begin council...</span>`
          }
        </div>

        <div class="sandstone-grid">${banners}</div>
      </div>
    `;

    document.getElementById("gpg-form").addEventListener("submit", (e) => {
      e.preventDefault();
      const next = document.getElementById("gpg-input").value.trim();
      if (!next) return;
      displayName = next;
      hasSetName = true;
      sessionStorage.setItem("matsya_custom_name", displayName);
      gpgNameLabel.textContent = displayName;
      sendCmd({ type: "update_name", display_name: displayName });
    });

    const beginBtn = document.getElementById("begin-council-btn");
    if (beginBtn) {
      beginBtn.addEventListener("click", () => sendCmd({ type: "start_game" }));
    }
  }

  function roleCardImage(role) {
    const map = {
      kanka: "./assets/images/kanka.webp",
      brihannala: "./assets/images/brihannala.webp",
      ballava: "./assets/images/ballava.webp",
      syrandhri: "./assets/images/sairandhri.webp",
      granthika: "./assets/images/granthika.webp",
      tantipala: "./assets/images/tantipala.webp",
      king_susharma: "./assets/images/susharma.webp",
      chief_spy: "./assets/images/infiltrator.webp",
      kichaka: "./assets/images/kichaka.webp",
      karna: "./assets/images/karna.webp",
    };
    return map[role] || "./assets/images/token_compass_star.png";
  }

  function renderDisguiseReveal(view) {
    const role = view.private_role;
    const factionCls = role.faction === "dharma" ? "faction-dharma" : "faction-infiltrators";
    const factionLabel = role.faction === "dharma" ? "Dharma Defenders" : "Infiltrators (Trigarta & Hastinapura)";

    mainStage.innerHTML = `
      <div class="card disguise-card needs-action ${factionCls}">
        <div class="action-cue-badge">Your Action Required</div>
        <span class="faction-pill">${factionLabel}</span>
        <img src="${roleCardImage(role.role)}" alt="${escapeHtml(role.display_name)}" class="character-portrait" />
        <h2 class="screen-title" style="font-size:1.8rem;">${escapeHtml(role.display_name)}</h2>
        <p style="color: var(--sandstone); margin-bottom: 10px;">${escapeHtml(role.title)}</p>
        <div class="vision-box">${escapeHtml(role.vision_summary)}</div>
        <p style="color: var(--text-muted); font-size:0.86rem; margin-bottom: 18px;">
          Private to your screen only. Never share your screen with other players.
        </p>
        <button id="understand-btn" class="btn btn-gold">I UNDERSTAND</button>
      </div>
    `;

    document.getElementById("understand-btn").addEventListener("click", () => {
      peekingDisguise = false;
      if (!role.acknowledged) {
        sendCmd({ type: "acknowledge_disguise" });
      } else {
        render();
      }
    });
  }

  function renderCouncilProposal(view) {
    const battle = view.battles[view.current_battle_index];
    const isLeader = view.leader_id === view.self_id;
    const leader = view.players.find((p) => p.id === view.leader_id);

    const rosterHtml = view.players
      .map((p) => {
        const isSelected = selectedTeamIds.has(p.id);
        const isLead = p.id === view.leader_id;
        return `
          <div class="roster-tile ${isLeader ? "selectable" : ""} ${isSelected ? "selected" : ""}" data-pid="${escapeHtml(p.id)}">
            <div>
              <strong>${escapeHtml(p.display_name)}</strong>
              ${p.id === view.self_id ? '<span class="tag tag-you" style="margin-left:6px;">You</span>' : ""}
            </div>
            <div style="display:flex; align-items:center; gap:6px;">
              ${isLead ? '<img src="./assets/images/icon_royal_signet.png" class="signet-icon" title="Council Leader" />' : ""}
              ${isSelected ? '<img src="./assets/images/token_compass_star.png" class="token-icon" title="Selected" />' : ""}
            </div>
          </div>
        `;
      })
      .join("");

    mainStage.innerHTML = `
      ${renderBattlesTrack(view)}
      <div class="card ${isLeader ? "needs-action" : "passive-wait"}">
        ${isLeader ? '<div class="action-cue-badge">Your Action Required &bull; Royal Signet</div>' : ""}
        <h2 class="screen-title">War Council Board &bull; Battle ${battle.number}: ${escapeHtml(battle.name)}</h2>
        <p class="screen-desc">
          ${
            isLeader
              ? `You hold the Royal Signet. Select exactly <strong>${battle.team_size}</strong> warriors for this battle.`
              : `<strong>${escapeHtml(leader?.display_name || "Leader")}</strong> holds the Royal Signet and is selecting <strong>${battle.team_size}</strong> warriors.`
          }
        </p>
        <div class="roster-grid">${rosterHtml}</div>
        ${
          isLeader
            ? `<div class="action-row">
                <button id="propose-team-btn" class="btn btn-gold" ${
                  selectedTeamIds.size === battle.team_size ? "" : "disabled"
                }>PROPOSE TEAM (${selectedTeamIds.size}/${battle.team_size})</button>
              </div>`
            : ""
        }
      </div>
    `;

    if (isLeader) {
      document.querySelectorAll(".roster-tile[data-pid]").forEach((el) => {
        el.addEventListener("click", () => {
          const pid = el.getAttribute("data-pid");
          if (selectedTeamIds.has(pid)) {
            selectedTeamIds.delete(pid);
          } else if (selectedTeamIds.size < battle.team_size) {
            selectedTeamIds.add(pid);
          }
          render();
        });
      });

      document.getElementById("propose-team-btn").addEventListener("click", () => {
        sendCmd({ type: "propose_team", team_ids: Array.from(selectedTeamIds) });
      });
    }
  }

  function renderCourtVote(view) {
    const me = view.players.find((p) => p.id === view.self_id);
    const hasVoted = me?.has_voted;
    const teamNames = view.proposed_team
      .map((id) => view.players.find((p) => p.id === id)?.display_name || id)
      .join(", ");
    const waiting = view.waiting_for_names.length
      ? `Waiting for ${view.waiting_for_names.join(", ")}...`
      : "Tallying ballots...";

    mainStage.innerHTML = `
      ${renderBattlesTrack(view)}
      <div class="card ${!hasVoted ? "needs-action" : "passive-wait"}">
        ${!hasVoted ? '<div class="action-cue-badge">Your Action Required &bull; Cast Vote</div>' : ""}
        <h2 class="screen-title">Court Vote</h2>
        <p class="screen-desc">
          Proposed Battle Team: <strong style="color: var(--gold);">${escapeHtml(teamNames)}</strong>
        </p>
        ${
          hasVoted
            ? `<div class="vision-box">Your vote is locked in. <strong>${escapeHtml(waiting)}</strong></div>`
            : `<div class="action-row">
                <button id="vote-yes-btn" class="btn btn-emerald">Approve (YES)</button>
                <button id="vote-no-btn" class="btn btn-crimson">Reject (NO)</button>
              </div>`
        }
      </div>
    `;

    if (!hasVoted) {
      document.getElementById("vote-yes-btn").addEventListener("click", () => {
        sendCmd({ type: "submit_vote", approve: true });
      });
      document.getElementById("vote-no-btn").addEventListener("click", () => {
        sendCmd({ type: "submit_vote", approve: false });
      });
    }
  }

  function renderBallotReveal(view) {
    const b = view.ballot_results;
    const me = view.players.find((p) => p.id === view.self_id);
    const canAdvance = view.leader_id === view.self_id || me?.is_host;

    const rows = b.entries
      .map(
        (e) => `
        <tr>
          <td>${escapeHtml(e.display_name)}</td>
          <td class="${e.approve ? "vote-yes" : "vote-no"}">${e.approve ? "APPROVE (YES)" : "REJECT (NO)"}</td>
        </tr>
      `
      )
      .join("");

    mainStage.innerHTML = `
      ${renderBattlesTrack(view)}
      <div class="card ${canAdvance ? "needs-action" : "passive-wait"}">
        ${canAdvance ? '<div class="action-cue-badge">Your Action Required</div>' : ""}
        <h2 class="screen-title">Ballot Reveal</h2>
        <p class="screen-desc">
          Result: <strong style="color:${b.approved ? "var(--emerald-bright)" : "var(--crimson-bright)"};">
            ${b.approved ? "APPROVED — Team Marches to Battle!" : "REJECTED — Royal Signet Passes Clockwise"}
          </strong> (${b.yes_count} YES / ${b.no_count} NO)
        </p>
        <table class="ballot-table">
          <thead><tr><th>Courtier</th><th>Vote</th></tr></thead>
          <tbody>${rows}</tbody>
        </table>
        ${
          canAdvance
            ? `<button id="advance-ballot-btn" class="btn btn-gold">${
                b.approved ? "BEGIN SECRET BATTLE" : "NEXT COUNCIL PROPOSAL"
              }</button>`
            : `<p style="color: var(--text-muted);">Waiting for Council Leader or Host to proceed...</p>`
        }
      </div>
    `;

    const btn = document.getElementById("advance-ballot-btn");
    if (btn) {
      btn.addEventListener("click", () => sendCmd({ type: "advance_from_ballot" }));
    }
  }

  function renderSecretBattle(view) {
    const battle = view.battles[view.current_battle_index];
    const progress = view.battle_progress;
    const action = view.private_battle_action;

    let bodyHtml = "";
    if (action) {
      if (action.has_submitted) {
        bodyHtml = `
          <div class="vision-box">
            Your battle card is sealed. Waiting for remaining warriors...
          </div>
        `;
      } else {
        bodyHtml = `
          <div class="privacy-banner">Keep your screen private! Choose your secret battle card:</div>
          <div class="battle-cards-row">
            <button id="card-defend-btn" class="battle-card-btn">
              <img src="./assets/images/card_protect_succeed.png" alt="Defend" />
              <strong>Defend (Succeed)</strong>
            </button>
            <button id="card-sabotage-btn" class="battle-card-btn" ${action.can_sabotage ? "" : "disabled"}>
              <img src="./assets/images/card_hinder_fail.png" alt="Sabotage" />
              <strong>Sabotage (Fail)</strong>
              ${action.can_sabotage ? "" : '<span style="font-size:0.75rem; color:var(--text-muted);">Dharma cannot Sabotage</span>'}
            </button>
          </div>
        `;
      }
    } else {
      bodyHtml = `
        <div class="vision-box">
          The Battle Team is acting in secret. Waiting for ${escapeHtml(
            progress.waiting_for_names.join(", ") || "warriors"
          )}...
        </div>
      `;
    }

    const needsBattleAction = Boolean(action && !action.has_submitted);
    mainStage.innerHTML = `
      ${renderBattlesTrack(view)}
      <div class="card ${needsBattleAction ? "needs-action" : "passive-wait"}">
        ${needsBattleAction ? '<div class="action-cue-badge">Your Action Required &bull; Secret Battle</div>' : ""}
        <h2 class="screen-title">Secret Battle &bull; ${escapeHtml(battle.name)}</h2>
        <p class="screen-desc">
          <strong>${progress.submitted_count} of ${progress.total_count} warriors have acted</strong>
        </p>
        ${bodyHtml}
      </div>
    `;

    const defendBtn = document.getElementById("card-defend-btn");
    if (defendBtn) {
      defendBtn.addEventListener("click", () => {
        sendCmd({ type: "submit_battle_action", card: "defend" });
      });
    }
    const sabBtn = document.getElementById("card-sabotage-btn");
    if (sabBtn && action?.can_sabotage) {
      sabBtn.addEventListener("click", () => {
        sendCmd({ type: "submit_battle_action", card: "sabotage" });
      });
    }
  }

  function renderBattleOutcome(view) {
    const outcome = view.latest_battle_outcome;
    const battle = view.battles[view.current_battle_index];
    const me = view.players.find((p) => p.id === view.self_id);
    const canAdvance = view.leader_id === view.self_id || me?.is_host;

    const cardsHtml = outcome.revealed_cards
      .map((c) => {
        const isDef = c === "defend";
        return `
          <div class="revealed-card-tile">
            <img src="./assets/images/${isDef ? "card_protect_succeed.png" : "card_hinder_fail.png"}" alt="${c}" />
            <div><strong>${isDef ? "Defend" : "Sabotage"}</strong></div>
          </div>
        `;
      })
      .join("");

    mainStage.innerHTML = `
      ${renderBattlesTrack(view)}
      <div class="card ${canAdvance ? "needs-action" : "passive-wait"}">
        ${canAdvance ? '<div class="action-cue-badge">Your Action Required</div>' : ""}
        <h2 class="screen-title">Battle Outcome &bull; ${escapeHtml(battle.name)}</h2>
        <p class="screen-desc">
          <strong style="color:${
            outcome.winner === "dharma" ? "var(--emerald-bright)" : "var(--crimson-bright)"
          };">
            ${outcome.winner === "dharma" ? "DHARMA DEFENDED THIS BATTLE!" : "INFILTRATORS SABOTAGED THIS BATTLE!"}
          </strong>
          (${outcome.defend_count} Defend, ${outcome.sabotage_count} Sabotage)
        </p>
        <div class="battle-cards-row">${cardsHtml}</div>
        ${
          canAdvance
            ? `<div class="action-row" style="justify-content:center;">
                <button id="advance-outcome-btn" class="btn btn-gold">CONTINUE COUNCIL</button>
              </div>`
            : `<p style="text-align:center; color: var(--text-muted);">Waiting for Council Leader or Host to continue...</p>`
        }
      </div>
    `;

    const btn = document.getElementById("advance-outcome-btn");
    if (btn) {
      btn.addEventListener("click", () => sendCmd({ type: "advance_from_battle_outcome" }));
    }
  }

  function renderFinalUnmasking(view) {
    const unmask = view.private_unmasking;
    if (unmask.is_susharma) {
      const targetsHtml = unmask.targets
        .map(
          (t) => `
          <button class="btn btn-crimson unmask-target-btn" data-tid="${escapeHtml(t.id)}">
            Unmask ${escapeHtml(t.display_name)} as Kanka
          </button>
        `
        )
        .join("");
      mainStage.innerHTML = `
        ${renderBattlesTrack(view)}
        <div class="card disguise-card faction-infiltrators needs-action">
          <div class="action-cue-badge">Your Action Required &bull; Final Unmasking</div>
          <h2 class="screen-title">Final Unmasking</h2>
          <p class="screen-desc">
            Dharma has won 3 battles, but you have one final chance! Choose who you believe is <strong>Kanka</strong>.
          </p>
          <div class="action-row" style="justify-content:center;">${targetsHtml}</div>
        </div>
      `;
      document.querySelectorAll(".unmask-target-btn").forEach((btn) => {
        btn.addEventListener("click", () => {
          sendCmd({ type: "submit_unmasking", target_id: btn.getAttribute("data-tid") });
        });
      });
    } else {
      mainStage.innerHTML = `
        ${renderBattlesTrack(view)}
        <div class="card disguise-card passive-wait">
          <h2 class="screen-title">Final Unmasking</h2>
          <div class="vision-box">
            <strong>King Susharma is searching the court for Kanka...</strong>
          </div>
          <p style="color: var(--text-muted);">Hold your breath — the fate of Matsya hangs in the balance.</p>
        </div>
      `;
    }
  }

  function renderVictory(view) {
    const v = view.victory;
    const me = view.players.find((p) => p.id === view.self_id);
    const isDharma = v.winner === "dharma";

    const rolesHtml = v.revealed_roles
      .map(
        (r) => `
        <div class="sandstone-banner" style="background:${
          r.faction === "dharma"
            ? "linear-gradient(155deg, #214e3a, #173326)"
            : "linear-gradient(155deg, #5e1b24, #2b1320)"
        }; color: var(--text-light); border-color:${
          r.faction === "dharma" ? "var(--emerald-bright)" : "var(--crimson-bright)"
        };">
          <div style="display:flex; align-items:center; gap:10px;">
            <img src="${roleCardImage(r.role)}" alt="${escapeHtml(r.role_name)}" class="revealed-portrait" />
            <div>
              <div class="player-name">${escapeHtml(r.display_name)}</div>
              <div style="font-size:0.84rem; color: var(--gold);">${escapeHtml(r.role_name)} &bull; ${escapeHtml(r.title)}</div>
            </div>
          </div>
          <span class="tag ${r.faction === "dharma" ? "tag-you" : "tag-host"}">${
          r.faction === "dharma" ? "Dharma" : "Infiltrator"
        }</span>
        </div>
      `
      )
      .join("");

    mainStage.innerHTML = `
      ${renderBattlesTrack(view)}
      <div class="card">
        <div style="text-align:center; margin-bottom:16px;">
          <img src="./assets/images/${isDharma ? "icon_crown_defender.png" : "icon_shield_rival.png"}" style="width:68px; height:68px;" />
          <h2 class="screen-title" style="font-size:1.8rem; margin-top:8px; color:${
            isDharma ? "var(--emerald-bright)" : "var(--crimson-bright)"
          };">${escapeHtml(v.headline)}</h2>
          <p class="screen-desc">${escapeHtml(v.summary)}</p>
        </div>
        <div class="sandstone-grid">${rolesHtml}</div>
        ${
          me?.is_host
            ? `<div style="text-align:center;">
                <button id="play-again-btn" class="btn btn-gold">Play Again</button>
              </div>`
            : `<p style="text-align:center; color: var(--text-muted);">Waiting for Host to start a new council...</p>`
        }
      </div>
    `;

    const btn = document.getElementById("play-again-btn");
    if (btn) {
      btn.addEventListener("click", () => sendCmd({ type: "play_again" }));
    }
  }

  function render() {
    if (!currentView) return;
    renderChat(currentView.chat_messages || []);

    const me = currentView.players.find((p) => p.id === currentView.self_id);
    if (me) {
      gpgNameLabel.textContent = me.display_name;
    }

    const showPeek =
      currentView.phase !== "lobby" &&
      currentView.phase !== "victory" &&
      currentView.private_role &&
      currentView.private_role.acknowledged;
    disguisePeekBtn.classList.toggle("hidden", !showPeek);

    if (currentView.phase === "lobby") {
      renderLobby(currentView);
      return;
    }

    if (
      currentView.private_role &&
      (!currentView.private_role.acknowledged || peekingDisguise) &&
      currentView.phase !== "victory"
    ) {
      renderDisguiseReveal(currentView);
      return;
    }

    switch (currentView.phase) {
      case "council_proposal":
        renderCouncilProposal(currentView);
        break;
      case "court_vote":
        renderCourtVote(currentView);
        break;
      case "ballot_reveal":
        renderBallotReveal(currentView);
        break;
      case "secret_battle":
        renderSecretBattle(currentView);
        break;
      case "battle_outcome":
        renderBattleOutcome(currentView);
        break;
      case "final_unmasking":
        renderFinalUnmasking(currentView);
        break;
      case "victory":
        renderVictory(currentView);
        break;
    }
  }

  function renderRoomGate() {
    mainStage.innerHTML = `
      <div class="card needs-action">
        <div class="action-cue-badge">Enter the Sabha</div>
        <h2 class="screen-title">Enter King Virata's Sabha</h2>
        <p class="screen-desc">
          Create a new 4-character court room or enter a room code shared by your host. You will enter your name after joining the room.
        </p>

        <div class="room-gate-grid">
          <div class="room-gate-box">
            <h3>Host a New Court</h3>
            <p style="color:var(--text-muted); font-size:0.88rem;">Generate a fresh 4-character room code and invite 5–10 courtiers.</p>
            <button id="create-room-btn" class="btn btn-gold" type="button">Create New Room</button>
          </div>

          <form id="join-room-form" class="room-gate-box">
            <h3>Join Existing Court</h3>
            <p style="color:var(--text-muted); font-size:0.88rem;">Enter the 4-character room code:</p>
            <input id="room-code-input" class="room-code-input" type="text" maxlength="4" placeholder="ABCD" autocomplete="off" />
            <button type="submit" class="btn btn-emerald">Join Room</button>
          </form>
        </div>
      </div>
    `;

    document.getElementById("create-room-btn").addEventListener("click", () => {
      const code = generateRoomCode();
      startAsHost(code);
    });

    document.getElementById("join-room-form").addEventListener("submit", (e) => {
      e.preventDefault();
      const code = document.getElementById("room-code-input").value.toUpperCase().replace(/[^A-Z0-9]/g, "");
      if (code.length !== 4) {
        showToast("Please enter a valid 4-character room code.");
        return;
      }
      joinAsGuest(code);
    });
  }

  if (roomCode) {
    joinAsGuest(roomCode);
  } else {
    renderRoomGate();
  }
})();
