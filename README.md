<div align="center">
  <h1>🛡️ AegiSec2</h1>
  <p><strong>Advanced Security Telemetry & Active Containment Platform</strong></p>
  
  [![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
  [![Rust](https://img.shields.io/badge/Agent-Rust_Tokio-orange.svg?logo=rust)]()
  [![Java](https://img.shields.io/badge/Backend-Java_21_WebFlux-green.svg?logo=spring)]()
  [![React](https://img.shields.io/badge/Dashboard-React_TS-blue.svg?logo=react)]()
</div>

<hr />

## 📖 Deep Architecture Overview

**AegiSec2** is not just a passive monitoring tool; it is an **active defense platform**. 

It combines a zero-overhead, low-level **Rust Agent** capable of deep Linux kernel introspection with an asynchronous, highly concurrent **Java 21 (Spring WebFlux)** backend. This allows AegiSec2 to detect anomalies, score risk dynamically, and execute automatic containment actions (like process tree killing and network isolation) within milliseconds.

---

## 🦀 1. The Agent (Rust)
Located in `/agent/src`, the agent is a lightning-fast daemon built on `tokio` for non-blocking I/O. It interfaces directly with Linux internals (`procfs`, `nix`, `inotify`) to gather telemetry without the overhead of traditional polling.

### 🧠 Dynamic Risk Scoring (DRS) Engine (`drs.rs`)
The heart of the agent's threat detection. Instead of relying purely on static signatures, it maintains an in-memory `HashMap` of active entities (UIDs) and calculates risk dynamically.

- **Stateful Behavioral Engine**: The DRS evaluates entities continuously, transitioning them across states (`Clean`, `Suspicious`, `Blocked`) based on an aggregated, dynamically computed risk vector.
- **Process Lineage Analysis [H-01, H-05]**: Audits hierarchical relationships in real-time. If it detects anomalous chains—such as a web service (`nginx`, `php`) or a script interpreter (`python`) spawning interactive shells (`bash`) with network-binding flags—it instantly shifts the entity to a critical risk state and mandates automated containment.
- **Temporal Density Tracking [H-02]**: Instead of blocking administrative commands outright, the engine tracks execution velocity. A sudden burst of reconnaissance tools (`whoami`, `netstat`, `id`) invoked by the same `UID` within a tight sliding window (e.g., 60 seconds) is algorithmically identified as an enumeration pattern, exponentially raising the risk profile.
- **High-Fidelity Signatures [H-00, H-04]**: Deterministic matching for known offensive artifacts (`msfvenom`, `socat`) and unauthorized outbound C2 ports (e.g., 4444, 1337).
- **FIM (File Integrity Monitoring)**: Uses `inotify` to track critical file modifications, dispatching immediate priority alerts.
- **Continuous Mathematical Decay**: To prevent static threshold saturation from benign anomalies, an asynchronous task applies a mathematical decay function every 60 seconds. Isolated events without follow-up activity gradually "cool down," safely returning the entity's risk profile to a baseline and dynamically minimizing false positives.

### 🔪 Active Containment Isolator (`isolator.rs`)
When the DRS Engine reaches critical thresholds, it triggers the Isolator module to physically stop the threat.

- **Surgical Process Tree Killing (`kill_tree`)**: Performs a breadth-first traversal of `/proc/<pid>/stat` to find all descendants of a rogue process. It sends `SIGSTOP` to freeze the entire tree, followed by `SIGKILL` from the leaves up to the root. This prevents malware from spawning orphan persistence processes while being terminated.
- **Network Blocking (`block_by_uid`, `block_ip_outbound`)**: Automatically cuts off Command & Control (C2) connections at the kernel level. It attempts to use modern `nftables` and features an automatic fallback to `iptables` to drop `OUTPUT` traffic matching the compromised UID or Destination IP.

### 📡 Telemetry Subsystem (`telemetry.rs`)
Designed to guarantee delivery of critical alerts even under heavy load.
- **Dual-Channel Queue**:
  - **Priority Queue**: Dedicated channel for containment events and critical alerts. Bypasses all buffers for an immediate HTTP POST.
  - **Normal Queue**: Batches standard telemetry and flushes every `N` seconds or when the batch size hits 50, minimizing network overhead.
- **Zero-Trust Security**: Enforces **mTLS** (Mutual TLS) using `reqwest` and `rustls` to ensure that only cryptographically verified agents can submit data to the backend.

---

## ☕ 2. The Central Server (Java 21 / Spring Boot)
Located in `/server/backend`, the core backend is designed to handle thousands of concurrent agents without thread blocking.

- **Reactive Core**: Utilizes Project Reactor (`spring-boot-starter-webflux`) for asynchronous, event-driven HTTP processing.
- **Real-time WebSockets**: Employs `WebSocket` + `STOMP` protocols to instantly push telemetry and alerts from the agents directly to the Dashboard UI, enabling zero-refresh monitoring.
- **Security**: Secured with Spring Security and stateless `JWT` (JSON Web Tokens) for authenticating dashboard administrators.
- **Persistence**: Leverages Spring Data JPA with a **PostgreSQL** database to store historical metrics, risk profiles, and audit logs.

---

## ⚛️ 3. The Command Dashboard (React / Vite)
Located in `/server/dashboard`, the frontend provides the "single pane of glass" for administrators.
- **Modern Stack**: Built with **React**, **TypeScript**, and **Vite** for blazing fast HMR and optimized builds.
- **Real-Time UI**: Subscribes to the backend's WebSocket topics to render metrics, risk scores, and isolation events as they happen live.
- **Styling**: Utilizes **TailwindCSS** for a responsive, dark-mode native interface.

---

## 🚀 Deployment (Quick Start)

AegiSec2's server components are fully containerized.

### 1. Start the Server Infrastructure
```bash
git clone https://github.com/your-username/AegiSec2.git
cd AegiSec2/server
docker-compose up -d
```
This spins up PostgreSQL, the Spring Boot WebFlux Backend, and the React Dashboard.

### 2. Access the Dashboard
Navigate to `http://localhost:5173` (or your configured port).

### 3. Deploy the Agent
Compile and run the Rust agent on the target Linux system (Requires `rustup`):
```bash
cd AegiSec2/agent
cargo build --release
sudo ./target/release/aegisec-agent
```
*(Root access is mandatory for the agent to perform deep `procfs` traversal and `nftables` isolation).*

---
## 📄 License
This project is licensed under the MIT License - see the LICENSE file for details.
