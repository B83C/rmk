# Configuration system design

This doc contains the thought of the RMK configuration system design.

## Background

The goal of RMK's configuration system is to provide users an easy and accessible way to set up keyboards (with or without Rust).

Apparently, a config file could be better for more people who don't know Rust, but we also want to keep some flexibility for customizing keyboard with Rust code.

## How?

There are 3 choices right now:

- [`cfg-toml`](https://github.com/jamesmunns/toml-cfg)
  - pros: 
    - a widely used lib
    - could overwrite default configs defined in RMK
    - easy to use 
  - cons:
    - need to add extra annotations to all config structs
    - some fields are not support
    - hard to expand to other types, accepts only numbers/strings in toml

- `build.rs`: Load the config in `build.rs`, then generate Rust code, which could be passed to RMK as config struct
  - pros:
    - Extendable, flexible, can do everything
    - No extra dependency
    - Need to access RMK config at build time
  - cons:
    - Need to distribute `build.rs`, users cannot use the lib without this file, which is not a common way generally
    - LOTS OF work

- Rust's procedural macro: add a macro like `#[rmk_keyboard]` and generate everything in compile-time
  - pros:
    - Extendable, flexible, and powerful, proc-macro can do everything
    - No need to distribute `build.rs`
    - Possible to make user's usage even much simpler
  - cons:
    - LOTS LOTS OF MACRO work
    - Complex, hard to maintain
    - Developing proc-macro might become a barrier for people who want to contribute to RMK

Okay, I'll pick the third approach(for now): writing proc-macros for RMK's configuration system. It brings simplicity for end-users but adds complexity to developers. RMK should consider users experience as the most important thing, over the developer experience(if they cannot be satisfied simultaneously), that's why I'd try proc-macro first.


## Problems

Besides the above choosing, there's some other problems that have to be addressed.

1. The first one is, how to deserialize those configs to RMK Config? 
   1. Using serde would be a way, but it requires some other annotations on RMK Config structs(may cause extra flash usage? TODO: test it)
   2. ✅ Another way is to define every field in config and convert then to RMK Config struct by hand. Seems to be a lot of works, but it's one-time investment.

2. The second problem is, how to convert different representations of GPIOs of different chips? For example, STMs have something like `PA1`, `PB2`, `PC3`, etc. nRFs have `P0_01`, ESPs have `gpio1`, rp2040 has `PIN_1`. Do we need a common representation of those different pin names? Or we just save strings in toml and process them differently.

    - ✅ proc_macro can do this

3. There are some other peripherals are commonly used in keyboards, such as spi, i2c, pwm and adc. There are some HAL traits for spi/i2c, so they're good. But for adc, there is no common trait AFAIK. For example, in `embassy-nrf`, it's called `SAADC` and it does not impl any external trait! How to be compatible with so many peripherals?
    - To be addressed
    - Temporary solution: add initializations code per chip

4. What if the config in toml is conflict with feature gate in `Cargo.toml`? Move some of the configs to `Cargo.toml`, or put them all in config file and update feature gate by config?
    - ✅ bin project's `Cargo.toml` can be loaded and parsed at compile-time


## Procedural macro approach

### Generated code

With proc-macro, the whole main function can be generated. But the main function varies between different chips. We have to separate the boilerplate code to several parts, making sure that the proc-macro won't become a mess. 

#### Before main function

There are some code out of main function, usually they should be placed before the main. Here is a list that RMK's proc-macro should add:

1. imports: yeah it's needed of course! And, it's actually quite complex, need to be carefully generated ensuring that no extra imports are added.

2. static configs: keyboard config, vial config, number of rows, etc

3. `bind_interrupts`: Embassy need this, it's complex too. The interrupt name and bind periphral names are actually something quite random, according to the chip

#### Embassy main

In the main function, generally there are several parts:

1. Chip initialization, with config:
    ```rust
      let mut config = Config::default();
      {
          use embassy_stm32::rcc::*;
          config.rcc.hse = Some(Hse {
              freq: Hertz(25_000_000),
              mode: HseMode::Oscillator,
          });
          config.rcc.pll_src = PllSource::HSE;
          config.rcc.pll = Some(Pll {
              prediv: PllPreDiv::DIV25,
              mul: PllMul::MUL192,
              divp: Some(PllPDiv::DIV2), // 25mhz / 25 * 192 / 2 = 96Mhz.
              divq: Some(PllQDiv::DIV4), // 25mhz / 25 * 192 / 4 = 48Mhz.
              divr: None,
          });
          config.rcc.ahb_pre = AHBPrescaler::DIV1;
          config.rcc.apb1_pre = APBPrescaler::DIV2;
          config.rcc.apb2_pre = APBPrescaler::DIV1;
          config.rcc.sys = Sysclk::PLL1_P;
      }

      // Initialize peripherals
      let p = embassy_stm32::init(config);
    ```

2. (Optional)USB peripheral initialization: just as what I wrote above, it's quite random!

    ```rust
      // It's STM32H7's USB initialization code
      static EP_OUT_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
      let mut usb_config = embassy_stm32::usb_otg::Config::default();
      usb_config.vbus_detection = false;
      let driver = Driver::new_fs(
          p.USB_OTG_HS,
          Irqs,
          p.PA12,
          p.PA11,
          &mut EP_OUT_BUFFER.init([0; 1024])[..],
          usb_config,
      );

      // It's nRF52840's USB initialization code in USB mode
      let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

      // It's nRF52840's USB initialization code in USB + BLE mode
      let software_vbus = SOFTWARE_VBUS.get_or_init(|| SoftwareVbusDetect::new(true, false));
      let driver = Driver::new(p.USBD, Irqs, software_vbus);

      // It's rp2040's USB initialization code
      let driver = Driver::new(p.USB, Irqs);
    ```

3. Storage initialization

4. Other keyboard config: Initialize `RmkConfig`, which contains usb config, vial config, ble_battery_config, etc.

5. Run the keyboard: RMK provides several functions to run the keyboard right now. The different entry function requires different inputs. The number of entry functions should be controller to a reasonable amount(I think not more than 3 variants is good). 