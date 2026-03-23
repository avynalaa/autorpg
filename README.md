# AutoRPG

A desktop text RPG driven by a large language model. AutoRPG acts as a persistent, stateful game engine: the AI narrates the world, and a deterministic Rust backend handles all mechanics — inventory, combat, quests, NPC relationships, currency, and world state — so the story never contradicts itself.

The application works with any OpenAI-compatible or Anthropic-compatible API endpoint, including hosted models (Anthropic, OpenAI, OpenRouter) and local inference servers (Ollama, LM Studio).

---

## Features

- **AI narration** — the model writes immersive second-person prose; all game-state changes are issued through a structured command protocol stripped before display
- **Character creation** — name, race, class, and backstory fed into the engine and AI context on every turn
- **Inventory system** — typed item categories (Weapon, Armor, Consumable, Quest, Tool, Key, Misc) with equip support
- **Currency** — four denominations: copper, silver, gold, platinum
- **Combat** — turn-based with attack/defence resolution and HP tracking
- **Quest log** — active, completed, and failed quests with per-quest objectives
- **NPC relations** — relationship scores tracked per NPC, surfaced in AI context
- **World map** — locations with type classification and connections, auto-built as the AI moves the player
- **Memory system** — recent events summarised and injected into each AI call so context stays coherent over long sessions
- **Conversation history** — configurable by turn count or estimated token count
- **Configurable response length** — max tokens per AI response is a user setting, not a hardcoded value
- **Save / load** — up to multiple named save slots with full game state and AI history
- **Theme switcher** — Slate, Codex, and Wire visual themes
- **Configurable system prompt** — the AI's instruction set is editable and resettable from the Settings panel
- **Provider-agnostic** — auto-detects Anthropic vs. OpenAI format from the endpoint URL, or set it manually

---

## Requirements

| Dependency | Minimum version |
|---|---|
| Rust | 1.77.2 |
| Node.js | 18 |
| npm | 9 |
| Tauri CLI | 2.x |

An API key for your chosen LLM provider is required at runtime.

---

## Building from Source

```sh
# Install Node dependencies
npm install

# Development (hot-reload frontend + Tauri window)
npm run tauri dev

# Production build
npm run tauri build
```

The compiled application bundle will be placed in `src-tauri/target/release/bundle/`.

---

## Configuration

All settings are accessible through the in-app Settings panel and persisted to disk between sessions.

| Setting | Description |
|---|---|
| API Endpoint | Base URL of your LLM provider |
| API Format | `auto`, `anthropic`, or `openai` |
| API Key | Stored in memory only, never written to disk |
| Model | Model ID string (or select from a fetched list) |
| System Prompt | The instruction given to the AI before every session |
| Conversation History | Limit by turns or estimated tokens |
| Max Response Tokens | Maximum tokens the AI may generate per response |

---

## Architecture

```
src/                  # SvelteKit frontend (Svelte 5, TypeScript)
src-tauri/src/
  ai/                 # API client (Anthropic + OpenAI), response parser, system prompt
  commands/           # Tauri command handlers, AI dispatch, game-state wiring
  engine/             # Pure game logic: character, combat, inventory, quests,
                      #   currency, map, memory, relations, dice
  settings.rs         # Persistent settings serialisation
  saves.rs            # Save / load slot management
```

The frontend never touches game state directly. All mutations go through Tauri commands, which parse the AI response for `[CMD:...]` directives, apply them to the engine, and return the cleaned narrative plus the updated state.

---

## License

See [LICENSE](LICENSE).
