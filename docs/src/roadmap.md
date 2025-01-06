# Roadmap

There are a bunch of things to do with RMK in the near future. I plan to ship 1.0.0 after all the following items are accomplished.

## Roadmap to 1.0.0

| Mark | Description |
| ---- | ----------- |
|  🔴  | important   |
|  🟢  | easy        |
|  🔵  | heavy work  |


#### keyboard feature
  - [x] layer support
  - [x] system/media/mouse keys
  - [x] LED
  - [x] tap/hold
  - [x] keyboard macros
  - [x] async key detection and report sending
  - [x] 🔵 split keyboard support
  - [x] Direct pin
  - [ ] 🔴 RGB
  - [ ] 🟢 encoder
  - [ ] 🔵 Input device
  - [ ] 🔵 display support

#### Wireless
  - [x] BLE support - nRF
  - [x] auto switch between BLE/USB
  - [x] battery service from ADC
  - [x] 🔴 BLE support - esp32c3 and esp32s3
  - [x] sleep mode to save battery
  - [ ] 🔵 universal BLE wrapper, including BLE management, battery management, supports both nRF and ESP
  - [ ] stablizing BLE feature gate/API
  - [ ] BLE support - ch58x/ch59x

#### User experience
  - [x] vial support
  - [x] easy keyboard configuration with good default, support different MCUs
  - [ ] Versioned documentation site, better documentation
  - [ ] CLI and GUI tool for project generation, firmware compilation, etc
  - [ ] making vial and default keymap consistent automatically
  - [ ] 🔴🔵 GUI keymap configurator which supports windows/macos/linux/web
  - [ ] default bootloader
  - [ ] USB DFU/OTA

If you want to contribute, please feel free to open an issue or PR, or just ping me! Any forms of contribution are welcome :D
