# Map Game - LLM-Powered Alternate History Grand Strategy

An interactive desktop grand strategy game where an LLM serves as the simulation engine for alternate history scenarios. Select a country, issue directives, and watch as the AI-driven world reacts — complete with wars, diplomacy, economy, and territorial changes rendered on a live world map.

## Features

- **LLM-Driven Simulation** — The entire world reacts dynamically to your decisions through a configurable LLM backend
- **Interactive World Map** — Real GeoJSON-based map with MapLibre GL, showing country borders, territories, and conflict zones
- **WW2 1939 Scenario** — Pre-built starting scenario with 37 countries, historical stats, and the European war in progress
- **12 Callable Tools** — The LLM can declare wars, mobilize forces, negotiate trade, form alliances, pass policies, and more
- **War Resolution System** — Combat simulated with military power, terrain modifiers, morale, technology, and luck
- **Economy Simulation** — GDP growth affected by war strain, mobilization, trade, and stability
- **Strategic Advisor** — Optional AI advisor that refines your directives with strategic context and risk assessment
- **Save/Load** — SQLite persistence with multiple save slots
- **Configurable LLM** — Support for OpenAI-compatible APIs and local models via Ollama

## Screenshot

```
┌─────────────────────────────────────────────────────┐
│  [Country Selector]  Scenario Info  [Settings]      │
├────────────────────────────────────┬────────────────┤
│                                    │  [Active Wars] │
│                                    ├────────────────┤
│      Interactive World Map         │                │
│      (MapLibre GL + GeoJSON)      │   Chat History  │
│                                    │                │
│                                    ├────────────────┤
│                                    │ [Advisor Panel]│
│                                    ├────────────────┤
│                                    │ [Command Input]│
├────────────────────────────────────┴────────────────┤
│  Germany | Pop: 79M | GDP: $450B | Army ████ Navy ██ Air ████ | Stability 75% │
└─────────────────────────────────────────────────────┘
```

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/) (stable toolchain)
- Platform-specific dependencies:
  - **Windows**: WebView2 (pre-installed on Windows 10+)
  - **Linux**: `libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev` and others
  - **macOS**: Xcode command line tools

### Installation

```bash
# Clone the repository
git clone https://github.com/YOUR_USER/map-game.git
cd map-game

# Install dependencies
npm install

# Run in development mode
npx tauri dev

# Build for production
npx tauri build
```

### LLM Configuration

The game requires an LLM backend to function. Two providers are supported:

1. **OpenAI-compatible** — Any API that follows the OpenAI chat completions format:
   - OpenAI (gpt-4, gpt-4o, etc.)
   - Anthropic (via compatible proxy)
   - Local servers like LM Studio, vLLM, etc.

2. **Ollama** — Local models running on your machine:
   - Install [Ollama](https://ollama.com/)
   - Pull a model: `ollama pull llama3`
   - Configure in-game with endpoint `http://localhost:11434`

Configure your provider in **Settings** (⚙ button in the top bar).

## How to Play

1. **Start a Scenario** — Select "ww2_1939" from the scenario screen and click Start
2. **Pick Your Country** — Use the search dropdown or click a country on the map
3. **Issue Directives** — Type commands in the chatbox like:
   - "Invade Poland with blitzkrieg tactics, focus all tank divisions on Warsaw"
   - "Negotiate a non-aggression pact with the Soviet Union"
   - "Increase military spending to 40% and mobilize for war"
   - "Seek an alliance with Italy and Japan"
4. **Use the Advisor** — Toggle the advisor to get strategic analysis before executing
5. **Advance Turns** — The world state updates after each directive, with wars progressing and economies shifting
6. **Watch the Map** — Territory changes, conflict zones, and diplomatic statuses update visually

## Architecture

```
┌──────────────────────────────────────────────────────┐
│  Tauri v2 Desktop Shell (Rust + WebView)             │
│                                                      │
│  ┌─────────────────────┐  ┌────────────────────────┐ │
│  │  Rust Backend        │  │  Frontend (React + TS) │ │
│  │                      │  │                        │ │
│  │  • LLM Providers     │  │  • MapLibre GL Map     │ │
│  │  • Game State Engine │  │  • Chatbox UI          │ │
│  │  • Tool Executors    │  │  • Country Selector    │ │
│  │  • Context Compressor│  │  • Status Dashboard    │ │
│  │  • Advisor System    │  │  • Settings Panel      │ │
│  │  • SQLite Persistence│  │  • Zustand State Mgmt  │ │
│  └─────────────────────┘  └────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop Shell | Tauri v2 |
| Frontend | React 18 + TypeScript + Vite |
| Map | MapLibre GL JS + Natural Earth GeoJSON |
| State Management | Zustand |
| Backend Logic | Rust |
| LLM Integration | reqwest (OpenAI-compatible API / Ollama) |
| Persistence | SQLite (rusqlite) |
| CI/CD | GitHub Actions |

## Project Structure

```
src-tauri/src/
├── main.rs              # App entry point, 14 Tauri commands
├── llm/                 # LLM provider abstraction
│   ├── provider.rs      # Trait + shared types
│   ├── openai.rs        # OpenAI-compatible provider
│   └── ollama.rs        # Ollama local provider
├── game/                # Core simulation engine
│   ├── country.rs       # Country struct + military/economic types
│   ├── state.rs         # GameState, events, turn history
│   ├── war.rs           # War, front, battle resolution
│   ├── diplomacy.rs     # Diplomatic actions and relations
│   └── economy.rs       # Economic processing
├── tools/               # LLM-callable tool system
│   ├── definitions.rs   # JSON schema for 12 tools
│   └── executors.rs     # Tool execution logic
├── context.rs           # System prompt + context builder
├── advisor.rs           # Strategic advisor system
└── db.rs                # SQLite save/load operations

src/                     # React frontend
├── components/          # UI components
├── stores/              # Zustand state
├── types/               # TypeScript types
└── utils/               # API wrappers + map utilities
```

## License

MIT — see [LICENSE](LICENSE) for details.
