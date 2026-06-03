# Básculas Compatibles - Fishr

## Puerto Serial Genérico
- Protocolo: RS232 / USB-Serial
- Baud rate: 9600 o 19200 (configurable)
- Data bits: 8
- Stop bits: 1
- Parity: None
- Flow control: None

### Modo Continuo
La báscula envía constantemente el peso en formato texto.

Formato común:
```
   1234 g
  +1234 g
  -1234 g
  1.234 kg
```

### Modo Comando
Enviar comando y recibir respuesta.

Comandos comunes:
- Solicitar peso: `W\r\n` o `P\r\n`
- Tara: `T\r\n` o `Z\r\n`
- Cero: `Z\r\n`

## Protocolos Probados

### Básculas CAS
- Modelos: PD, AP, SW
- Comando: Continuo
- Formato: `    0.000 kg`

### Básculas Torrey
- Modelos: L-EQ, LP-EQ, REQ
- Comando: Continuo o Peso bajo demanda
- Comando peso: `#\r\n`

### Básculas UWE
- Modelos: UW, WP
- Comando: Continuo
- Formato: `   1234g`

### Básculas Digi
- Modelos: D系列
- Comando: Continuo
- Formato: `    1.234kg`

## Configuración
En el archivo .env de la sucursal:
```
SCALE_PORT=/dev/ttyUSB0
SCALE_BAUD=9600
PRINTER_PORT=/dev/ttyUSB1
PRINTER_BAUD=19200
```
