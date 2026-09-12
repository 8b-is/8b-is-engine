# ternaryPureASCII — the enthea wire format (spec + protocol)

The surface layer of the enthea language: everything the machine speaks is
trits `{-1, 0, +1}`, so the wire is that, made transmissible in pure ASCII.

## The refactor

The shrine's retro table — a per-character base-3 lookup — is refactored
into a **generative codec**: no table, just balanced-ternary arithmetic.
The seed grew up: what was a list of letters is now a protocol.

## The alphabet

One trit, three pure-ASCII characters:

| trit | char |
|------|------|
| -1   | `-`  |
| 0    | `0`  |
| +1   | `+`  |

No other character appears in a frame (except the magic and separator).

## The encoding

Each byte `b` is centred on zero (`v = b - 128`, so -128..127) and written
as **six balanced trits**, least significant first, canonical (no digit 2
or -2). Balanced ternary covers -364..364, so any byte fits exactly.

## The universe of training's data plane

One alphabet everywhere — the machine's own trits:

| data | wire |
|------|------|
| MD (markdown) | `mq` — marqant token compression (`enthea compress`) |
| TEXT | `t3:` — ternaryPureASCII (this spec) |
| JSON | `t3j:` — **ternarySIMDJSON**: JSON marshalled, then the same six-trit
        bytes + 1-bit model, under a distinct magic so the decoder knows the
        payload is JSON |
| YAML | `t3y:` — **qYAML**: a YAML document framed on the same six-trit
        bytes + 1-bit model, under its own magic. The wire never parses the
        config; it carries it, judged |

`enthea wire` demonstrates TEXT; `EncodeJSON`/`DecodeJSON` demonstrate JSON.
The classroom's status reports travel as `t3j` frames — delayed but LIVE,
self-judged, in the alphabet the pupil is made of.

## qYAML — the config lane

Every configuration the classroom speaks — the council table, the pupil
settings, the lanes — is YAML, and YAML rides the same wire: `t3y:` frames a
document byte-identical, judged by the same 1-bit model. `EncodeYAMLText` /
`DecodeYAML` in Go; `encode_yaml_text` / `decode_yaml` in Python. The wire
never needs to understand the config; it carries it, self-judged.

## The frame

```
t3:<payload>:<checksum>
```

- `t3` — the magic (balanced ternary, the machine's own alphabet).
- `payload` — six trit-chars per byte.
- `checksum` — **one trit, and it is a 1-bit LLM.** Every frame carries its
  own judge: a fixed periodic ternary weight vector (the `checksumModel`)
  is dotted against the payload trits — `qdot` — and the dot is sharpened
  to a single trit — `ultra`. The verdict is the checksum.

## Properties

- **Pure ASCII** — the alphabet is `-`, `0`, `+`; the frame survives any
  7-bit channel.
- **Round-trip exact** — bytes in, bytes out, byte-identical.
- **Self-judging** — a single-trit flip at any nonzero weight shifts the
  dot by ±2w, and mod 3 that is never zero when w ≠ 0: the 1-bit model
  catches every nonzero-weight corruption. Zero-weight positions are its
  acknowledged blind spot.
- **The machine's own alphabet** — the same trits the VM, the bus, and the
  quant-ctx window use. The wire is not a translation of the language; it
  is the language, transmissible.
