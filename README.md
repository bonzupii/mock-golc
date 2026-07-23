# Mock GOLC Rig

A lightweight mock rig for testing lighting console output via Art-Net. Listens on UDP
port 6454 and renders fixture state in an Iced GUI.

## DMX Universe Layout

All fixtures below are on **Universe 0**. Addresses are 1-indexed.

| Address | Fixture      | Channels                                |
|---------|--------------|-----------------------------------------|
| 1       | Par 1        | Dimmer, Red, Green, Blue                |
| 5       | Par 2        | Dimmer, Red, Green, Blue                |
| 9       | Par 3        | Dimmer, Red, Green, Blue                |
| 13      | Par 4        | Dimmer, Red, Green, Blue                |
| 17      | Wash 1       | Dimmer, Red, Green, Blue, White         |
| 22      | Wash 2       | Dimmer, Red, Green, Blue, White         |
| 27      | Dimmer Rack  | Dimmer                                  |
| 28      | UV Wash      | Dimmer, Red, Green, Blue, UV            |
| 33      | Amber Par    | Dimmer, Amber                           |
| 35      | Strobe       | Dimmer, Custom                          |

- Custom channels are accepted but ignored by the mock.
- When the Dimmer channel is 0, state rendering treats it as 1.0 (full) so that color-only
  fixtures display correctly even with no explicit dimmer level.

## Fixture Profiles

Each fixture maps incoming DMX values to its channels. Channel values are normalised to
`0.0–1.0` internally.

- **RGB fixtures** (Pars 1–4): 4 channels each (3 addresses consumed by colour after dimmer).
- **RGB+White** (Washes 1–2): 5 channels; white is accepted but not rendered in the GUI
  (only RGB+dimmer contributes to the colour swatch).
- **UV Wash**: 5 channels; UV channel is accepted but not rendered.
- **Amber Par**: 2 channels; amber is accepted but not reflected in the GUI swatch.
- **Strobe**: 2 channels; the custom channel is a placeholder for strobe rate or similar —
  no special behaviour is implemented.

## State Display

Each fixture renders a 60×60 px colour swatch and info panel showing:

- Fixture name and DMX address
- The computed RGB colour (dimmer‑modulated) or greyscale dimmer value

The window title shows the active packet's sender IP and universe number after the first
packet is received.
