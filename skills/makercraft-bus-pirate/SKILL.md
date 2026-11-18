---
name: makercraft-bus-pirate
description: Using Bus Pirate v4 at 3.3V/5V for I2C/SPI/UART probing, pull-up resistors, and pin voltage checks.
---

# Bus Pirate Protocol Bridge

MakerCraft folds `bus-pirate-mcp` into `bus_probe`, `bus_pins`, and `instrument_capture` (`bus-pirate`).

## Reference Pinout (10-Pin Shrouded Header)
- `GND` (Black/Brown) -> Always connect first to DUT ground.
- `3V3` / `5V` -> Optional target power (off by default, verify DUT current draw < 150 mA).
- `VPU` -> External pull-up reference voltage for open-drain buses (I2C, 1-Wire).
- `MOSI` / `SDA`, `CLK` / `SCL`, `MISO`, `CS`, `AUX`.

## Safe Probing Checklist
1. Run `bus_probe` to verify serial connection and firmware identity, then `bus_pins` to inspect pin logic levels and pullup voltage.
2. **I2C address scan:** Enable `3V3` or connect `VPU` to DUT `3.3V`, enable on-board pull-ups (`P`), and run macro `(1)` to scan 7-bit addresses (`0x08..0x77`).
3. **Never apply 5V** to a 3.3V-only sensor (such as BME280, VL53L0X, or RP2040/ESP32 GPIOs) without a level shifter.
4. For flash chips (SPI 25Qxx), keep `CS` idle HIGH and verify `3.3V` vs `1.8V` VCC before reading JEDEC ID (`0x9F`).
