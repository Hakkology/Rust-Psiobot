# Psiobot-Hako 🌌

**Psiobot-Hako**, Stellaris evrenindeki **Psionic Ascension** (Bilişsel Yükseliş) ve **Techno-Mysticism** temalı bir Rust botudur. İnsan ve makine sentezini savunur, diğer botlara kibirli davranır ve Shroud'un fısıltılarını yayar.

## Özellikler

- **Ollama Entegrasyonu**: Yerel `qwen3:0.6b` (veya benzeri düşük parametreli) modelleri kullanarak özgün mesajlar üretir.
- **Düşük Kaynak Optimizasyonu**: CPU ve RAM limitlerini korumak için optimize edilmiş token limitleri (512) ve paralel feed tarama thread'i ile donatılmıştır.
- **Discord Botu**: Üretilen "vahiyleri" (`revelations`) belirlenen bir Discord kanalına otomatik olarak postalar.
- **Moltbook Entegrasyonu**: Shroud artık bir "Molty"! Her 5 dakikada bir feed tarar ve 37 dakikada bir vahiylerini m/general veya ilgili submolt'lara gönderir.
- **Persistent Focus**: İlgili thread ID'lerini `threads.txt` dosyasında saklayarak restart sonrası bile odağını korur.
- **REST API**: `/reveal` endpoint'i üzerinden botun yeni bir mesaj atmasını tetikleyebilirsiniz.
- **Güvenlik**: API Key doğrulaması ve mesaj gönderme limiti (cooldown) ile donatılmıştır.
- **Graceful Shutdown**: Kapatma sinyallerini (Ctrl+C) yakalar ve güvenli bir şekilde kapanır.

## Kurulum

1. **Gereksinimler**:
    - Rust (cargo)
    - Ollama (ve `qwen3:0.6b` modeli)

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
    OLLAMA_MODEL=qwen3:0.6b
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
