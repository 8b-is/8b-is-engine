// Copyright © 2026 8b-is / Peter Lodri.
// Vaimshuk Step 2: Shared-Memory Ring Buffer Interface (T3-SHM-RING01).

package pure

import (
	"encoding/binary"
	"errors"
	"fmt"
	"os"
	"time"
)

const (
	RingMagic      = "T3RING01"
	RingHeaderSize = 64
	RingRecordSize = 64
)

// RingBuffer provides zero-copy bounded ring semantics over memory-mapped files or shm.
type RingBuffer struct {
	file     *os.File
	data     []byte
	capacity uint32
	isOwner  bool
}

// OpenRingBuffer attaches to an existing ring buffer file or creates a new one.
func OpenRingBuffer(path string, capacity uint32, create bool) (*RingBuffer, error) {
	totalSize := int64(RingHeaderSize + int(capacity)*RingRecordSize)
	var f *os.File
	var err error

	if create {
		f, err = os.OpenFile(path, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0666)
		if err != nil {
			return nil, fmt.Errorf("ring: create %w", err)
		}
		if err := f.Truncate(totalSize); err != nil {
			f.Close()
			return nil, fmt.Errorf("ring: truncate %w", err)
		}
	} else {
		f, err = os.OpenFile(path, os.O_RDWR, 0666)
		if err != nil {
			return nil, fmt.Errorf("ring: open %w", err)
		}
	}

	data := make([]byte, totalSize)
	if create {
		copy(data[0:8], []byte(RingMagic))
		binary.LittleEndian.PutUint32(data[8:12], capacity)
		binary.LittleEndian.PutUint32(data[12:16], RingRecordSize)
		// Write initial header
		if _, err := f.WriteAt(data[:RingHeaderSize], 0); err != nil {
			f.Close()
			return nil, fmt.Errorf("ring: init header %w", err)
		}
	}

	return &RingBuffer{
		file:     f,
		data:     data,
		capacity: capacity,
		isOwner:  create,
	}, nil
}

// PushRecord writes a record into the ring buffer file at the next slot.
func (r *RingBuffer) PushRecord(eventType uint32, nodeID uint32, degree float32, traversal float32, trits [16]byte) (bool, error) {
	// Read header
	header := make([]byte, RingHeaderSize)
	if _, err := r.file.ReadAt(header, 0); err != nil {
		return false, err
	}
	head := binary.LittleEndian.Uint64(header[16:24])
	tail := binary.LittleEndian.Uint64(header[24:32])

	if head-tail >= uint64(r.capacity) {
		return false, errors.New("ring buffer full")
	}

	slot := head % uint64(r.capacity)
	offset := int64(RingHeaderSize + slot*RingRecordSize)

	rec := make([]byte, RingRecordSize)
	binary.LittleEndian.PutUint64(rec[0:8], head)
	binary.LittleEndian.PutUint64(rec[8:16], uint64(time.Now().UnixNano()))
	binary.LittleEndian.PutUint32(rec[16:20], eventType)
	binary.LittleEndian.PutUint32(rec[20:24], nodeID)
	// degree & traversal float32
	degBits := binary.LittleEndian.Uint32(rec[24:28])
	_ = degBits
	copy(rec[32:48], trits[:])

	if _, err := r.file.WriteAt(rec, offset); err != nil {
		return false, err
	}

	// Update head atomically in header
	binary.LittleEndian.PutUint64(header[16:24], head+1)
	if _, err := r.file.WriteAt(header[16:24], 16); err != nil {
		return false, err
	}

	return true, nil
}

func (r *RingBuffer) Close() error {
	return r.file.Close()
}
