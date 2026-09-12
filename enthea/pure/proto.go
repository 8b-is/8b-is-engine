// pure/proto.go — the pure-stdlib protobuf wire codec.
//
// The constellation's typed lane: every string has a struct, every struct
// has a protobuf format. This codec is the protobuf wire format written by
// hand — varints, tags, length-delimited fields — with zero dependencies,
// matching the enthea doctrine (pure stdlib, no generated code). The typed
// frames then travel on the ternary wire as t3p: (protobuf → balanced trits
// → pure ASCII), byte-identical round-trip, judged by the 1-bit model.
package pure

import (
	"fmt"
)

// ProtoField is one encoded protobuf field.
type ProtoField struct {
	Num  int    // field number (1..15 fit one tag byte; larger uses two)
	Wire int    // wire type: 0 = varint, 2 = length-delimited
	Var  uint64 // varint value (wire 0)
	Byt  []byte // bytes value (wire 2)
}

// appendVarint writes v as a base-128 varint.
func appendVarint(b []byte, v uint64) []byte {
	for v >= 0x80 {
		b = append(b, byte(v)|0x80)
		v >>= 7
	}
	return append(b, byte(v))
}

// readVarint reads a varint starting at b[i]; returns value and next index.
func readVarint(b []byte, i int) (uint64, int, error) {
	var v uint64
	var shift uint
	for {
		if i >= len(b) {
			return 0, i, fmt.Errorf("pure: proto: truncated varint")
		}
		x := b[i]
		i++
		v |= uint64(x&0x7f) << shift
		if x&0x80 == 0 {
			return v, i, nil
		}
		shift += 7
		if shift >= 64 {
			return 0, i, fmt.Errorf("pure: proto: varint overflow")
		}
	}
}

// EncodeProto encodes fields to the protobuf wire format.
func EncodeProto(fields []ProtoField) []byte {
	out := make([]byte, 0, 64)
	for _, f := range fields {
		if f.Num == 0 {
			continue
		}
		out = appendVarint(out, uint64(f.Num<<3|f.Wire))
		if f.Wire == 0 {
			out = appendVarint(out, f.Var)
		} else if f.Wire == 2 {
			out = appendVarint(out, uint64(len(f.Byt)))
			out = append(out, f.Byt...)
		}
	}
	return out
}

// DecodeProto parses protobuf wire format into fields.
func DecodeProto(b []byte) ([]ProtoField, error) {
	var fields []ProtoField
	i := 0
	for i < len(b) {
		tag, ni, err := readVarint(b, i)
		if err != nil {
			return nil, err
		}
		i = ni
		num := int(tag >> 3)
		wire := int(tag & 7)
		switch wire {
		case 0:
			v, ni, err := readVarint(b, i)
			if err != nil {
				return nil, err
			}
			i = ni
			fields = append(fields, ProtoField{Num: num, Wire: 0, Var: v})
		case 2:
			l, ni, err := readVarint(b, i)
			if err != nil {
				return nil, err
			}
			i = ni
			if i+int(l) > len(b) {
				return nil, fmt.Errorf("pure: proto: truncated bytes field %d", num)
			}
			fields = append(fields, ProtoField{Num: num, Wire: 2, Byt: append([]byte(nil), b[i:i+int(l)]...)})
			i += int(l)
		default:
			return nil, fmt.Errorf("pure: proto: unsupported wire type %d", wire)
		}
	}
	return fields, nil
}

// Field returns the first field with the given number, or nil.
func FieldsOf(fields []ProtoField, num int) *ProtoField {
	for i := range fields {
		if fields[i].Num == num {
			return &fields[i]
		}
	}
	return nil
}

// EncodeProtoFrame writes protobuf bytes on the ternary wire (t3p:).
func EncodeProtoFrame(proto []byte) string {
	return encodeFrame(proto, wireProtoMagic)
}

// DecodeProtoFrame parses a t3p: frame back to protobuf bytes.
func DecodeProtoFrame(s string) ([]byte, error) {
	return decodeFrame(s, wireProtoMagic)
}
