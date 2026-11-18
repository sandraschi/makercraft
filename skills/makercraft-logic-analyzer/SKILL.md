---
name: makercraft-logic-analyzer
description: Capturing digital buses with sigrok/SUMP logic analyzers, sample-rate rules, and I2C/SPI/UART/Step-Dir decoding.
---

# Logic Analyzer Guide

MakerCraft folds `logic-analyzer-mcp` into `logic_capture`, `logic_usb` (sigrok-cli), and `instrument_capture` (`logic-analyzer`).

## Sample Rate Rule (4x-10x)
- Sample at **at least 4x** (ideally **8x-10x**) the fastest signal frequency or baud rate:
  - 115200 baud UART -> >= 1 MHz sample rate
  - 400 kHz I2C -> >= 4 MHz sample rate
  - 8 MHz SPI -> >= 24-50 MHz sample rate
- Too low a sample rate causes aliased edges and false protocol decode errors.

## Wiring & Thresholds
- Always connect **at least one GND lead** (ideally one near high-speed clock lines) before clipping channel leads.
- Match logic threshold to the DUT (`3.3V CMOS` vs `5V TTL` vs `1.8V`). Cheap 24 MHz FX2LP clones expect `2.0V-5.25V` and are NOT 1.8V safe without level shifting.

## Common Maker Protocol Checks
- **SPI (CS, CLK, MOSI, MISO):** Verify CPOL/CPHA mode (Mode 0..3) and active-low CS framing.
- **I2C (SDA, SCL):** Check for missing pull-up resistors (idle must be HIGH) and ACK/NACK on address byte.
- **TMC2209 / Stepper UART:** Single-wire half-duplex 115200 baud; look for clean `0x05` sync byte and CRC.
