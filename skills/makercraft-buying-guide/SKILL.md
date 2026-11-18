---
name: makercraft-buying-guide
description: Practical buying advice for 3D printers, desktop CNCs, oscilloscopes, logic analyzers, multimeters, and debug probes.
---

# Maker Machine & Bench Instrument Buying Guide

## 1. 3D Printers (FDM)
- **Best turn-key / reliable:** Bambu Lab A1 / A1 Mini (budget) or P1S / X1C (enclosed ABS/ASA); Prusa MK4S / CORE One (open-source, repairable, EU support).
- **Best Klipper tinkerer:** Voron 0.2 / 2.4 / Trident (kit), Sovol SV06/SV08, or modded Ender-3 V3 KE.
- **Avoid:** Unenclosed bed-slingers with proprietary nozzles/nozzle-PCBs and zero community firmware, or ultra-cheap acrylic-frame kits.

## 2. Desktop CNC Routers
- **Light PCB / wood / acrylic (budget):** 3018-Pro / Genmitsu 3018-PROVer V2 (GRBL 1.1h, limit switches + Z-probe). Keep expectations realistic: softwood, plastics, FR4 PCBs only.
- **Aluminum & serious rigid milling:** RatRig Stronghold, Shapeoko 4/5, PrintNC (DIY steel), or Carvera / Makera for enclosed desktop.
- **Must-haves:** ER11/ER20 collets, emergency stop mushroom button, proper dust shoe + HEPA shop vac, and tramming/spoilboard plan.

## 3. Oscilloscopes
- **Best value 4-channel:** Rigol DHO804 / DHO914 (12-bit, USB-C PD, HDMI, SCPI over LAN/USBTMC) or Siglent SDS804X HD (12-bit, very clean UI/FFT).
- **Minimum spec:** 4 channels (essential for SPI), >= 70-100 MHz bandwidth, 12-bit ADC if doing power/sensor work.
- **Avoid:** 1-channel toy pocket scopes for serious debugging (no trigger correlation).

## 4. Logic Analyzers
- **Budget:** 24 MHz 8-ch Cypress FX2LP clone (~15 EUR) + **PulseView / sigrok** (great for UART/I2C/slow SPI at 3.3V-5V).
- **Pro / High-speed:** Saleae Logic Pro 8/16 (great software, analog+digital) or DSLogic Plus / U3Pro16 (sigrok-compatible, shielded leads, 100-400 MHz).

## 5. Multimeters & Soldering
- **DMM:** Brymen BM235 / BM869s, Fluke 117/87V, or UNI-T UT61E+ (always check **CAT III / CAT IV** input protection and ceramic HRC fuses if touching mains).
- **Soldering:** Pinecil V2 / TS101 (USB-C PD, great portable) or Hakko FX-888D / JBC C245 clone (Aixun T3A/T420D) for fast thermal recovery.
- **Protocol & Debug:** Bus Pirate 5 (modern RP2040, color screen, built-in logic/pullups) or v4; Raspberry Pi Debugprobe / CMSIS-DAP or ST-Link V3 / J-Link EDU Mini for SWD/JTAG.
