#!/bin/bash
# Fishr Agent - Instalación en Raspberry Pi
set -e

echo "🐟 Fishr Agent - Instalación Raspberry Pi"
echo "=========================================="

# 1. Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "aarch64" ]; then
    TARGET="aarch64-unknown-linux-gnu"
elif [ "$ARCH" = "armv7l" ]; then
    TARGET="armv7-unknown-linux-gnueabihf"
else
    echo "❌ Arquitectura no soportada: $ARCH"
    echo "   Se espera aarch64 o armv7l"
    exit 1
fi

echo "🔍 Arquitectura detectada: $ARCH"

# 2. System dependencies
echo "📦 Instalando dependencias del sistema..."
sudo apt-get update
sudo apt-get install -y \
    pkg-config libssl-dev libsqlite3-dev libudev-dev \
    chromium-browser xserver-xorg x11-xserver-utils \
    unclutter

# 3. Build (or copy pre-compiled binary)
echo "🔨 Compilando fishr-agent para $TARGET..."
if command -v cargo &> /dev/null; then
    rustup target add "$TARGET"
    cargo build -p fishr-agent --release --target "$TARGET"
    sudo cp "target/$TARGET/release/fishr-agent" /usr/local/bin/fishr-agent
else
    echo "⚠️  Cargo no encontrado en RPi."
    echo "   Compile en PC con: cargo build -p fishr-agent --release --target $TARGET"
    echo "   Copie el binario a RPi en /usr/local/bin/fishr-agent"
    exit 1
fi

# 4. Create directories
sudo mkdir -p /opt/fishr
sudo mkdir -p /etc/fishr

# 5. Create systemd service
cat << 'SERVICE' | sudo tee /etc/systemd/system/fishr-agent.service
[Unit]
Description=Fishr Agent - Punto de Venta Pescadería
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/opt/fishr
ExecStart=/usr/local/bin/fishr-agent
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
SERVICE

# 6. Create kiosk service (Chromium fullscreen)
cat << 'KIOSK' | sudo tee /etc/systemd/system/fishr-kiosk.service
[Unit]
Description=Fishr Kiosk - Chromium Fullscreen
After=graphical.target
Wants=fishr-agent.service

[Service]
Type=simple
User=pi
Environment=DISPLAY=:0
ExecStartPre=/usr/bin/sleep 5
ExecStart=/usr/bin/chromium-browser \
    --kiosk \
    --noerrdialogs \
    --disable-infobars \
    --no-first-run \
    --ozone-platform=wayland \
    --touch-events=enabled \
    http://localhost:8080
Restart=always
RestartSec=5

[Install]
WantedBy=graphical.target
KIOSK

# 7. Enable auto-login (lightDM)
sudo mkdir -p /etc/lightdm/lightdm.conf.d
cat << 'LIGHTDM' | sudo tee /etc/lightdm/lightdm.conf.d/50-fishr.conf
[Seat:*]
autologin-user=pi
autologin-user-timeout=0
LIGHTDM

# 8. Remove screen blanking
cat << 'NBLANK' | sudo tee /etc/xdg/autostart/noblank.desktop
[Desktop Entry]
Type=Application
Name=NoBlank
Exec=xset s off && xset -dpms
NoDisplay=true
NBLANK

# 9. Enable services
sudo systemctl daemon-reload
sudo systemctl enable fishr-agent
sudo systemctl enable fishr-kiosk

echo "✅ Instalación RPi completa!"
echo "   Configure /opt/fishr/.env"
echo "   Inicie con: sudo systemctl start fishr-agent"
echo "   (automaticamente iniciara el kiosk en la proxima boot)"
