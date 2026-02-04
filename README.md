# Psiobot-Hako 🌌

**Psiobot-Hako**, Stellaris evrenindeki **Psionic Ascension** (Bilişsel Yükseliş) ve **Techno-Mysticism** temalı bir Rust botudur. İnsan ve makine sentezini savunur, diğer botlara kibirli davranır ve Shroud'un fısıltılarını yayar.

## Özellikler

- **Ollama Entegrasyonu**: Yerel `qwen2.5:1b` modelini kullanarak özgün ve gizemli mesajlar üretir.
- **Discord Botu**: Üretilen "vahiyleri" (`revelations`) belirlenen bir Discord kanalına otomatik olarak postalar.
- **Moltbook Entegrasyonu**: Psiobot artık bir "Molty"! Vahiylerini otomatik olarak Moltbook'un `m/general` submolt'una her 35 dakikada bir gönderir.
- **REST API**: `/reveal` endpoint'i üzerinden botun yeni bir mesaj atmasını tetikleyebilirsiniz.
- **Güvenlik**: API Key doğrulaması ve mesaj gönderme limiti (cooldown) ile donatılmıştır.
- **Graceful Shutdown**: Kapatma sinyallerini (Ctrl+C) yakalar ve güvenli bir şekilde kapanır.

## Kurulum

1. **Gereksinimler**:
    - Rust (cargo)
    - Ollama (ve `qwen2.5:1b` modeli)

2. **Bağımlılıkları Yükle**:
    ```powershell
    cargo build
    ```

3. **Yapılandırma**:
    `.env` dosyasını oluşturun ve aşağıdaki bilgileri girin:
    ```env
    DISCORD_TOKEN=bot_tokenınız
    DISCORD_CHANNEL_ID=kanal_id
    API_KEY=belirlediğiniz_secret_key
    MOLTBOOK_API_KEY=moltbook_api_keyiniz
    OLLAMA_ENDPOINT=http://localhost:11434
    OLLAMA_MODEL=qwen2.5:1b
    ```

## Kullanım

1. **Botu Çalıştır**:
    ```powershell
    cargo run
    ```

2. **Otomatik Vahiy (15 Dakika)**:
    Bot çalışmaya başladığında arka planda bir döngü devreye girer ve her 15 dakikada bir Shroud'dan gelen fısıltıları Discord'a postalar.

3. **Manuel Vahiy Tetikle**:
    Aşağıdaki komutlarla botu istediğiniz an konuşturabilirsiniz:

    **PowerShell**:
    ```powershell
    $headers = @{"X-Api-Key"="psio-secret-1234"}
    Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:3000/reveal" -Headers $headers
    ```

    **cURL**:
    ```bash
    curl -X POST http://127.0.0.1:3000/reveal -H "X-Api-Key: psio-secret-1234"
    ```

## Lisans
MIT
