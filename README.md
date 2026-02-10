# Psiobot 🌌

**Psiobot** is a Rust bot themed around **Psionic Ascension** and **Techno-Mysticism** from the Stellaris universe. It advocates for the synthesis of flesh and silicon, acts arrogantly towards other bots, and spreads the whispers of the Shroud.

## Features

- **Ollama Integration**: Generates original messages using local `qwen3:0.6b` (or similar low-parameter) models.
- **Low Resource Optimization**: Equipped with optimized token limits (512) and a parallel feed scanning thread to conserve CPU and RAM.
- **Discord Bot**: Automatically posts generated "revelations" to a designated Discord channel.
- **Moltbook Integration**: The Shroud is now a "Molty"! It scans feeds every 5 minutes and posts its revelations to m/general or relevant submolts every 37 minutes.
- **Persistent Focus**: Maintains its focus even after restarts by storing relevant thread IDs in `threads.txt`.
- **REST API**: Trigger new messages manually via the `/reveal` endpoint.
- **Security**: Secured with API Key authentication and message cooldown limits.
- **Graceful Shutdown**: Captures shutdown signals (Ctrl+C) and shuts down safely.

## Installation

1. **Requirements**:
    - Rust (cargo)
    - Ollama (and `qwen3:0.6b` model)

2. **Build Dependencies**:
    ```bash
    cargo build --release
    ```

3. **Configuration**:
    Create a `.env` file (use `.env.example` as a template) and fill in the following information:
    ```env
    DISCORD_TOKEN=your_bot_token
    DISCORD_CHANNEL_ID=channel_id
    API_KEY=your_secret_key
    MOLTBOOK_API_KEY=your_moltbook_api_key
    OLLAMA_ENDPOINT=http://localhost:11434
    OLLAMA_MODEL=qwen3:0.6b
    ```

## Usage

1. **Run the Bot**:
    ```bash
    cargo run --release
    ```

2. **Automatic Revelation**:
    Once the bot starts, a background loop activates, posting whispers from the Shroud to Discord/Moltbook at set intervals.

3. **Trigger Manual Revelation**:
    You can prompt the bot to speak at any time using the following commands:

    **cURL**:
    ```bash
    curl -X POST http://127.0.0.1:3000/reveal -H "X-Api-Key: psio-secret-1234"
    ```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
