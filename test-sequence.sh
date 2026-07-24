#!/usr/bin/env python3
"""Send a test sequence of Art-Net frames to the mock rig."""
import socket, struct, time, sys, os

HOST = sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1"
PORT = int(sys.argv[2]) if len(sys.argv) > 2 else 6454
DELAY = float(sys.argv[3]) if len(sys.argv) > 3 else 0.5

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

def artdmx(seq, universe, data):
    header = struct.pack(
        "<8s2H2B2H",
        b"Art-Net\x00",
        0x5000,
        14,
        seq & 0xFF,
        0,
        universe & 0xFFFF,
        len(data) & 0xFFFF,
    )
    return header + bytes(data)

seq = 0

def send(label, data):
    global seq
    packet = artdmx(seq, 0, data)
    sock.sendto(packet, (HOST, PORT))
    print(f"  [{seq:02d}] {label}")
    seq += 1
    time.sleep(DELAY)

# Fixture addresses (1-indexed, DMX data is 0-indexed):
#   Par 1:  1-4   (dimmer, R, G, B)
#   Par 2:  5-8   (dimmer, R, G, B)
#   Par 3:  9-12  (dimmer, R, G, B)
#   Par 4:  13-16 (dimmer, R, G, B)
#   Wash 1: 17-21 (dimmer, R, G, B, W)
#   Wash 2: 22-26 (dimmer, R, G, B, W)
#   Mv 1:   27-35 (dimmer, pan, pan_fine, tilt, tilt_fine, cw, gw, shutter, zoom)
#   Mv 2:   36-44 (dimmer, pan, pan_fine, tilt, tilt_fine, cw, gw, shutter, zoom)

DMX = [0] * 512

def dmx(addr, *values):
    for i, v in enumerate(values):
        DMX[addr - 1 + i] = v & 0xFF

def pan_16bit(deg, p_range=(0, 540)):
    frac = (deg - p_range[0]) / (p_range[1] - p_range[0])
    return int(frac * 65535)

def tilt_16bit(deg, t_range=(0, 270)):
    frac = (deg - t_range[0]) / (t_range[1] - t_range[0])
    return int(frac * 65535)

print(f"Sending test sequence to {HOST}:{PORT} (delay {DELAY}s)\n")

# Step 1: Par 1 red
dmx(1, 255, 255, 0, 0)
send("Par 1 → full red", DMX)

# Step 2: Par 1 green
dmx(1, 255, 0, 255, 0)
send("Par 1 → full green", DMX)

# Step 3: Par 1 blue
dmx(1, 255, 0, 0, 255)
send("Par 1 → full blue", DMX)

# Step 4: Par 2 cyan
dmx(5, 255, 0, 255, 255)
send("Par 2 → full cyan", DMX)

# Step 5: Par 3 + Par 4 magenta + yellow
dmx(9, 255, 255, 0, 255)
dmx(13, 255, 255, 255, 0)
send("Par 3 magenta · Par 4 yellow", DMX)

# Step 6: Wash 1 white
dmx(17, 255, 0, 0, 0, 255)
send("Wash 1 → full white", DMX)

# Step 7: Wash 2 amber mix
dmx(22, 255, 255, 128, 0, 128)
send("Wash 2 → warm mix", DMX)

# Step 8: All pars off, movers on
dmx(1, 0, 0, 0, 0)
dmx(5, 0, 0, 0, 0)
dmx(9, 0, 0, 0, 0)
dmx(13, 0, 0, 0, 0)
dmx(17, 0, 0, 0, 0, 0)
dmx(22, 0, 0, 0, 0, 0)
p = pan_16bit(0); t = tilt_16bit(0)
dmx(27, 255, p >> 8, p & 0xFF, t >> 8, t & 0xFF, 32, 0, 255, 128)
p = pan_16bit(540); t = tilt_16bit(270)
dmx(36, 255, p >> 8, p & 0xFF, t >> 8, t & 0xFF, 64, 32, 255, 255)
send("Pars off · Mover 1→R,0° · Mover 2→G,540°/270°", DMX)

# Step 9: Mover 1 sweeps pan
for deg in range(0, 541, 45):
    DMX = [0] * 512
    dmx(27, 255, (p := pan_16bit(deg)) >> 8, p & 0xFF, (t := tilt_16bit(135)) >> 8, t & 0xFF, 96, 0, 255, 128)
    dmx(36, 255, 0, 0, 0, 0, 0, 0, 0, 0)
    send(f"Mover 1 → pan {deg}°, tilt 135°", DMX)

# Step 10: Mover 2 colour wheel cycle
for slot in (32, 64, 96, 128, 160, 192, 224):
    DMX = [0] * 512
    dmx(36, 255, 0, 0, 0, 0, slot, 0, 255, 128)
    send(f"Mover 2 → colour wheel slot {slot}", DMX)

# Step 11: All off
DMX = [0] * 512
send("All fixtures off", DMX)

# Step 12: Full intensity white on pars
dmx(1, 255, 255, 255, 255)
dmx(5, 255, 255, 255, 255)
dmx(9, 255, 255, 255, 255)
dmx(13, 255, 255, 255, 255)
dmx(17, 255, 255, 255, 255, 255)
dmx(22, 255, 255, 255, 255, 255)
send("All wash + pars → full white", DMX)

sock.close()
print(f"\nDone — {seq} frames sent.")
