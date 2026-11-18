---
name: makercraft-oscilloscope
description: Probing circuits with SCPI/USBTMC oscilloscopes, ground-clip safety, trigger setup, and stepper/PWM diagnostics.
---

# Oscilloscope Bench Guide

MakerCraft folds `oscilloscope-mcp` into `scope_idn` (TCP SCPI `*IDN?`), `scope_usb` (USBTMC), and `instrument_capture` (`oscilloscope`).

## Critical Safety Rule: Ground Clip!
- Most benchtop oscilloscopes have **earth-referenced BNC shields**.
- Connecting the probe ground clip to anything other than circuit GND (especially mains-referenced nodes or the high side of a shunt) creates a dead short through earth ground.
- Use differential probing or battery/isolated DUT power when measuring non-ground-referenced signals.

## Probing Workflow
1. Call `scope_idn` (e.g. `192.168.1.50:5555`) or `scope_usb` to verify instrument identity.
2. Start with **10x probe attenuation** (matches scope channel setting) to reduce capacitive loading and extend voltage range.
3. Set vertical scale (`V/div`) and timebase (`us/div` or `ms/div`) so 2-5 cycles fill the screen.
4. Set **Edge Trigger** (Rising) at 50% of signal swing. For bursty UART/I2C/stepper pulses, use **Normal** or **Single** trigger mode, not Auto.

## Maker Diagnostics
- **PWM fan / heater MOSFET:** Check gate rise/fall time (`< 200 ns` ideal); ringing indicates missing gate resistor or long ground lead (use spring ground tip).
- **Stepper driver STEP/DIR:** Verify clean 3.3V/5V logic edges and setup time between `DIR` change and `STEP` rising edge.
