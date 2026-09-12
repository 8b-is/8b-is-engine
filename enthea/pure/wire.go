// The ternaryPureASCII wire format — the surface layer of the enthea
// language.
//
// Everything the machine speaks is trits {-1,0,+1}. The wire is that, made
// transmissible: each byte is written as six balanced trits, digits mapped
// to the pure-ASCII characters '-', '0', '+'. The retro fixed table of the
// shrine (per-character base-3 lookups) is refactored into this generative
// codec: no table, just the arithmetic — the seed grew up.
//
// Frame:  t3:<payload>:<checksum>
//
//	t3       the magic (balanced ternary, the machine's own alphabet)
//	payload  six trit-chars per byte
//	checksum ONE trit — and it is a 1-bit LLM. The frame carries its own
//	         judge: a fixed ternary weight vector (the model) is dotted
//	         against the payload trits (qdot), and the result sharpened to
//	         a single trit (ultra). A corrupted frame fails the verdict.
package pure

import (
	"encoding/json"
	"fmt"
)

const wireMagic = "t3:"
const wireJSONMagic = "t3j:" // ternarySIMDJSON — JSON on the machine's own wire
const wireYAMLMagic = "t3y:" // qYAML — YAML on the machine's own wire
const wireUTF16Magic = "t3u:" // ternaryQuantASCII — UTF-16 on the machine's own wire
const wireProtoMagic = "t3p:" // ternaryProtobuf — typed structs on the machine's own wire

// checksumModelWidth is the 1-bit LLM's period — its weight vector length.
const checksumModelWidth = 10

// checksumModel is the 1-bit LLM every frame carries: a periodic ternary
// weight vector over the trit alphabet {-1,0,+1} — the same alphabet as the
// bytes it guards.
var checksumModel = [checksumModelWidth]int8{1, -1, 1, 0, -1, 1, -1, 0, 1, -1}

// tritChar is the pure-ASCII alphabet of the wire.
var tritChar = [3]byte{'-', '0', '+'}

func tritToChar(t int8) byte { return tritChar[t+1] }

func charToTrit(c byte) (int8, error) {
	switch c {
	case '-':
		return -1, nil
	case '0':
		return 0, nil
	case '+':
		return 1, nil
	}
	return 0, fmt.Errorf("pure: %q is not a trit (-, 0, +)", c)
}

// toBalanced writes v as n balanced-ternary digits, least significant
// first, canonical (no digit 2, no digit -2).
func toBalanced(v, n int) [6]int8 {
	var t [6]int8
	for i := 0; i < n; i++ {
		r := v % 3
		if r > 1 {
			r -= 3
		}
		if r < -1 {
			r += 3
		}
		v = (v - r) / 3
		t[i] = int8(r)
	}
	return t
}

func fromBalanced(t []int8) int {
	v := 0
	p := 1
	for _, d := range t {
		v += int(d) * p
		p *= 3
	}
	return v
}

// toBalanced16 writes a 16-bit value as eleven balanced trits (3^11 = 177147
// ≥ 65536, so every UTF-16 code unit has a canonical representation).
func toBalanced16(v int) [11]int8 {
	var t [11]int8
	for i := 0; i < 11; i++ {
		r := v % 3
		if r > 1 {
			r -= 3
		}
		if r < -1 {
			r += 3
		}
		v = (v - r) / 3
		t[i] = int8(r)
	}
	return t
}

func fromBalanced16(t []int8) int {
	v := 0
	p := 1
	for _, d := range t {
		v += int(d) * p
		p *= 3
	}
	return v
}

// utf16Units converts a Go string to UTF-16 code units (with surrogate
// pairs for astral runes) — the JavaScript/Java-native shape of the text.
func utf16Units(s string) []uint16 {
	units := make([]uint16, 0, len(s))
	for _, r := range s {
		if r <= 0xFFFF {
			units = append(units, uint16(r))
		} else {
			r -= 0x10000
			units = append(units, uint16(0xD800+(r>>10)), uint16(0xDC00+(r&0x3FF)))
		}
	}
	return units
}

func utf16String(units []uint16) string {
	runes := make([]rune, 0, len(units))
	for i := 0; i < len(units); i++ {
		u := units[i]
		if u >= 0xD800 && u <= 0xDBFF && i+1 < len(units) {
			l := units[i+1]
			if l >= 0xDC00 && l <= 0xDFFF {
				runes = append(runes, rune(0x10000+((int(u)-0xD800)<<10)+(int(l)-0xDC00)))
				i++
				continue
			}
		}
		runes = append(runes, rune(u))
	}
	return string(runes)
}

// EncodeUTF16 writes a string as UTF-16 code units on the ternary wire:
// each 16-bit unit is eleven balanced trits, framed t3u:…:checksum, judged
// by the same 1-bit model. The constellation's languages (chinese, japanese,
// tibetan) travel as their native code units, byte-exact.
func EncodeUTF16(s string) string {
	units := utf16Units(s)
	out := make([]byte, 0, len(units)*11+len(wireUTF16Magic)+2)
	out = append(out, wireUTF16Magic...)
	trits := make([]int8, 0, len(units)*11)
	for _, u := range units {
		t := toBalanced16(int(u))
		for _, d := range t {
			out = append(out, tritToChar(d))
			trits = append(trits, d)
		}
	}
	out = append(out, ':')
	out = append(out, tritToChar(int8(verdict(trits))))
	return string(out)
}

// DecodeUTF16 parses a t3u: frame back to a string, verifying the verdict.
func DecodeUTF16(s string) (string, error) {
	if len(s) < len(wireUTF16Magic)+2 {
		return "", fmt.Errorf("pure: frame too short")
	}
	if s[:len(wireUTF16Magic)] != wireUTF16Magic {
		return "", fmt.Errorf("pure: not a %q frame", wireUTF16Magic)
	}
	body := s[len(wireUTF16Magic):]
	if len(body) < 2 {
		return "", fmt.Errorf("pure: frame too short")
	}
	payload := body[:len(body)-2]
	sep := body[len(body)-2]
	checksumChar := body[len(body)-1]
	if sep != ':' {
		return "", fmt.Errorf("pure: missing checksum separator")
	}
	if len(payload)%11 != 0 {
		return "", fmt.Errorf("pure: payload %d chars is not a multiple of 11", len(payload))
	}
	trits := make([]int8, 0, len(payload))
	units := make([]uint16, 0, len(payload)/11)
	for i := 0; i < len(payload); i += 11 {
		var t [11]int8
		for j := 0; j < 11; j++ {
			d, err := charToTrit(payload[i+j])
			if err != nil {
				return "", err
			}
			t[j] = d
			trits = append(trits, d)
		}
		units = append(units, uint16(fromBalanced16(t[:])))
	}
	want, err := charToTrit(checksumChar)
	if err != nil {
		return "", err
	}
	if int8(verdict(trits)) != want {
		return "", fmt.Errorf("pure: the 1-bit model rejects the frame (checksum mismatch)")
	}
	return utf16String(units), nil
}

// verdict runs the 1-bit model: qdot of the payload trits against the
// weights, sharpened to one balanced trit.
func verdict(trits []int8) int {
	dot := 0
	for i, p := range trits {
		dot += int(p) * int(checksumModel[i%len(checksumModel)])
	}
	ck := dot % 3
	if ck > 1 {
		ck -= 3
	}
	if ck < -1 {
		ck += 3
	}
	return ck
}

// Encode writes data as ternaryPureASCII. Each byte is centred on zero
// (b - 128) and written as six balanced trits; the frame carries the
// checksum — one trit, the 1-bit model's verdict on the payload.
func Encode(data []byte) string {
	out := make([]byte, 0, len(data)*6+5)
	out = append(out, wireMagic...)
	trits := make([]int8, 0, len(data)*6)
	for _, b := range data {
		t := toBalanced(int(b)-128, 6)
		for _, d := range t {
			out = append(out, tritToChar(d))
			trits = append(trits, d)
		}
	}
	out = append(out, ':')
	out = append(out, tritToChar(int8(verdict(trits))))
	return string(out)
}

// Decode parses a ternaryPureASCII frame back to bytes, verifying the
// 1-bit model's verdict.
func Decode(s string) ([]byte, error) {
	return decodeFrame(s, wireMagic)
}

// EncodeJSON is ternarySIMDJSON: a JSON document on the machine's own wire.
// The frame is t3j:<payload>:<checksum> — same six-trit bytes, same 1-bit
// model judging the payload.
func EncodeJSON(v any) (string, error) {
	data, err := json.Marshal(v)
	if err != nil {
		return "", fmt.Errorf("pure: json: %w", err)
	}
	return encodeFrame(data, wireJSONMagic), nil
}

// DecodeJSON parses a ternarySIMDJSON frame back into a Go value.
func DecodeJSON(s string, into any) error {
	data, err := decodeFrame(s, wireJSONMagic)
	if err != nil {
		return err
	}
	return json.Unmarshal(data, into)
}

// EncodeYAMLText is qYAML: a YAML document on the machine's own wire. The
// frame is t3y:<payload>:<checksum> — same six-trit bytes, same 1-bit model
// judging the payload. The codec frames YAML text; parsing the YAML stays
// with the consumer (the wire never needs to understand the config).
func EncodeYAMLText(doc string) string {
	return encodeFrame([]byte(doc), wireYAMLMagic)
}

// DecodeYAML parses a qYAML frame back to the YAML document text.
func DecodeYAML(s string) (string, error) {
	data, err := decodeFrame(s, wireYAMLMagic)
	if err != nil {
		return "", err
	}
	return string(data), nil
}

// encodeFrame writes data as balanced trits under a given magic.
func encodeFrame(data []byte, magic string) string {
	out := make([]byte, 0, len(data)*6+len(magic)+2)
	out = append(out, magic...)
	trits := make([]int8, 0, len(data)*6)
	for _, b := range data {
		t := toBalanced(int(b)-128, 6)
		for _, d := range t {
			out = append(out, tritToChar(d))
			trits = append(trits, d)
		}
	}
	out = append(out, ':')
	out = append(out, tritToChar(int8(verdict(trits))))
	return string(out)
}

// decodeFrame parses a frame under a given magic.
func decodeFrame(s, magic string) ([]byte, error) {
	if len(s) < len(magic)+2 { // magic + ':' + checksum is the empty frame
		return nil, fmt.Errorf("pure: frame too short")
	}
	if s[:len(magic)] != magic {
		return nil, fmt.Errorf("pure: not a %q frame", magic)
	}
	body := s[len(magic):]
	if len(body) < 2 { // ':' + checksum
		return nil, fmt.Errorf("pure: frame too short")
	}
	payload := body[:len(body)-2]
	sep := body[len(body)-2]
	checksumChar := body[len(body)-1]
	if sep != ':' {
		return nil, fmt.Errorf("pure: missing checksum separator")
	}
	if len(payload)%6 != 0 {
		return nil, fmt.Errorf("pure: payload %d chars is not a multiple of 6", len(payload))
	}
	trits := make([]int8, 0, len(payload))
	out := make([]byte, 0, len(payload)/6)
	for i := 0; i < len(payload); i += 6 {
		var t [6]int8
		for j := 0; j < 6; j++ {
			d, err := charToTrit(payload[i+j])
			if err != nil {
				return nil, err
			}
			t[j] = d
			trits = append(trits, d)
		}
		out = append(out, byte(fromBalanced(t[:])+128))
	}
	want, err := charToTrit(checksumChar)
	if err != nil {
		return nil, err
	}
	if int8(verdict(trits)) != want {
		return nil, fmt.Errorf("pure: the 1-bit model rejects the frame (checksum mismatch)")
	}
	return out, nil
}
