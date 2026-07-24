# Mock GOLC Rig

A virtual Art-Net lighting rig for testing console output. Listens on UDP port 6454
and renders fixture state in a responsive Iced GUI.

## Features

- **Multi-universe** — tab bar to switch between universes
- **Fixture profiles** — RGB par, RGBW wash, moving head with pan/tilt and colour/gobo wheels
- **Responsive grid** — fixture cards that wrap to fit the window
- **Stage view** — 2D top-down plot with beam direction for movers and cursor hover highlight
- **Config editor** — add/remove universes, fixtures, and profiles from a built-in panel; saved to `~/.config/mock-golc/rig.toml`
- **Dark/light theme** toggle

## Default Rig (Universe 0)

| Address | Name     | Profile      |
|---------|----------|--------------|
| 1       | Par 1    | RGB Par      |
| 5       | Par 2    | RGB Par      |
| 9       | Par 3    | RGB Par      |
| 13      | Par 4    | RGB Par      |
| 17      | Wash 1   | RGBW Wash    |
| 22      | Wash 2   | RGBW Wash    |
| 27      | Mover 1  | Moving Head  |
| 36      | Mover 2  | Moving Head  |

### Built-in Profiles

| Profile       | Channels                                                  |
|---------------|-----------------------------------------------------------|
| RGB Par       | Dimmer, Red, Green, Blue                                  |
| RGBW Wash     | Dimmer, Red, Green, Blue, White                           |
| Moving Head   | Dimmer, Pan, Pan Fine, Tilt, Tilt Fine, Color Wheel, Gobo Wheel, Shutter, Zoom |

Moving heads include colour and gobo wheel slot definitions (8 colour positions,
5 gobo positions) and a 540° pan / 270° tilt range.

## Usage

Run the binary; it binds `0.0.0.0:6454` and waits for Art-Net ArtDmx packets.
Switch between grid and stage view with the buttons in the header bar. Open the
config panel to edit the rig at runtime — changes persist to `rig.toml`.

When the Dimmer channel is 0, the mock treats it as 1.0 (full) so that colour-only
fixtures display correctly even without an explicit dimmer level.
