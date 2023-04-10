# Introduction to embedded Rust

This is the source code for my talk "Introduction to embedded Rust" at the [Capitole du Libre](https://capitoledulibre.org/) conference in November 2022.

Support material: [slides](https://docs.google.com/presentation/d/e/2PACX-1vQQf8JCeoIuUm98XDuzg1yzfgfku3OcY-W9JL_1Rhw-FaMXPpdGU93jeDcCD2Q7RMvajOdt1hipcXp7/pub) and video [in French](https://www.youtube.com/watch?v=2SxO74QofRA) and [in English](https://www.youtube.com/watch?v=pDoOPl5ptGs).

There are two projects:

* `intro-attiny`: a "blink" demo for an attiny85, with a variable rate driven by a LDR (light dependent resistor)
* `intro-esp32`: two demos for an ESP32
  * Display the value of an LDR on a tiny OLED display.
  * Display the value of an LDR on a self-refreshing web page with an embedded http server powered by ESP-IDF.

Useful links:

* [Presentation slides](https://docs.google.com/presentation/d/e/2PACX-1vQQf8JCeoIuUm98XDuzg1yzfgfku3OcY-W9JL_1Rhw-FaMXPpdGU93jeDcCD2Q7RMvajOdt1hipcXp7/pub).
* [Rust Embedded main page](https://github.com/rust-embedded).
* [Rust on AVR devices](https://github.com/avr-rust/).
* [Rust on ESP devices](https://github.com/esp-rs).