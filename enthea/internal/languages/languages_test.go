package languages

import "testing"

func TestAllLanesRoundTrip(t *testing.T) {
	for _, l := range All() {
		ok, err := RoundTrip(l)
		if err != nil {
			t.Fatalf("%s: %v", l.Name, err)
		}
		if !ok {
			t.Errorf("%s: wire round-trip mismatch", l.Name)
		}
	}
}

func TestBridgePairDistinct(t *testing.T) {
	es, hu := BridgePair()
	if es == "" || hu == "" {
		t.Fatal("bridge lanes must be present")
	}
	if es == hu {
		t.Error("esperanto and hungarian frames must differ")
	}
}

func TestByName(t *testing.T) {
	if _, err := ByName("esperanto"); err != nil {
		t.Fatalf("esperanto lane missing: %v", err)
	}
	if _, err := ByName("magyar"); err != nil {
		t.Fatalf("hungarian native name missing: %v", err)
	}
	if _, err := ByName("klingon"); err == nil {
		t.Fatal("unknown lane must error")
	}
}

func TestLanguageProtoRoundTrip(t *testing.T) {
	for _, l := range All() {
		back, err := FromProto(l.Proto())
		if err != nil {
			t.Fatalf("%s: proto decode: %v", l.Name, err)
		}
		if back.Name != l.Name || back.Native != l.Native || back.Phrase != l.Phrase ||
			back.Agglutin != l.Agglutin || back.Wire != l.Wire {
			t.Errorf("%s: proto round-trip mismatch", l.Name)
		}
	}
}

func TestLanguageProtoFrameRoundTrip(t *testing.T) {
	for _, l := range All() {
		back, err := FromProtoFrame(l.ProtoFrame())
		if err != nil {
			t.Fatalf("%s: proto frame decode: %v", l.Name, err)
		}
		if back.Phrase != l.Phrase {
			t.Errorf("%s: proto frame phrase mismatch", l.Name)
		}
	}
}
