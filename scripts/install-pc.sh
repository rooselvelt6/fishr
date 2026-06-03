#!/bin/bash
# Fishr Agent - Instalación en PC Linux
set -e

echo "🐟 Fishr Agent - Instalación PC"
echo "================================"

# 1. Check Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust no instalado. Instale: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 2. Check system deps
echo "📦 Instalando dependencias del sistema..."
sudo apt-get update
sudo apt-get install -y pkg-config libssl-dev libsqlite3-dev libudev-dev

# 3. Build agent
echo "🔨 Compilando fishr-agent..."
cargo build -p fishr-agent --release

# 4. Copy binary
sudo cp target/release/fishr-agent /usr/local/bin/fishr-agent

# 5. Create config directory
sudo mkdir -p /etc/fishr

# 6. Create systemd service
cat << 'SERVICE' | sudo tee /etc/systemd/system/fishr-agent.service
[Unit]
Description=Fishr Agent - Punto de Venta Pescadería
After=network.target

[Service]
Type=simple
User=fishr
WorkingDirectory=/opt/fishr
ExecStart=/usr/local/bin/fishr-agent
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=FISHR_DB=/opt/fishr/fishr.db

[Install]
WantedBy=multi-user.target
SERVICE

# 7. Create user and directory
sudo useradd -r -s /bin/false fishr || true
sudo mkdir -p /opt/fishr
sudo chown fishr:fishr /opt/fishr

# 8. Enable service
sudo systemctl daemon-reload
sudo systemctl enable fishr-agent

echo "✅ Instalación completa!"
echo "   Configure /opt/fishr/.env y ejecute: sudo systemctl start fishr-agent"
