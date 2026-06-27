package main

import (
	"sync"
)

type EmitterType int

const (
	EmitterGaussian EmitterType = iota
	EmitterVoidZone
)

type SpatialRegistry struct {
	mu       sync.RWMutex
	Active   []bool
	FreeList []int
	X        []float32
	Y        []float32
	Z        []float32
	VX       []float32
	VY       []float32
	VZ       []float32
	AX       []float32
	AY       []float32
	AZ       []float32
	Mass     []float32
}

func NewSpatialRegistry() *SpatialRegistry {
	return &SpatialRegistry{
		Active:   make([]bool, 0),
		FreeList: make([]int, 0),
		X:        make([]float32, 0),
		Y:        make([]float32, 0),
		Z:        make([]float32, 0),
		VX:       make([]float32, 0),
		VY:       make([]float32, 0),
		VZ:       make([]float32, 0),
		AX:       make([]float32, 0),
		AY:       make([]float32, 0),
		AZ:       make([]float32, 0),
		Mass:     make([]float32, 0),
	}
}

func (sr *SpatialRegistry) AddEntity(x, y, z, mass float32) int {
	sr.mu.Lock()
	defer sr.mu.Unlock()

	idx := -1
	if len(sr.FreeList) > 0 {
		idx = sr.FreeList[len(sr.FreeList)-1]
		sr.FreeList = sr.FreeList[:len(sr.FreeList)-1]
	} else {
		idx = len(sr.X)
		sr.Active = append(sr.Active, false)
		sr.X = append(sr.X, 0)
		sr.Y = append(sr.Y, 0)
		sr.Z = append(sr.Z, 0)
		sr.VX = append(sr.VX, 0)
		sr.VY = append(sr.VY, 0)
		sr.VZ = append(sr.VZ, 0)
		sr.AX = append(sr.AX, 0)
		sr.AY = append(sr.AY, 0)
		sr.AZ = append(sr.AZ, 0)
		sr.Mass = append(sr.Mass, 0)
	}

	sr.Active[idx] = true
	sr.X[idx] = x
	sr.Y[idx] = y
	sr.Z[idx] = z
	sr.VX[idx] = 0
	sr.VY[idx] = 0
	sr.VZ[idx] = 0
	sr.AX[idx] = 0
	sr.AY[idx] = 0
	sr.AZ[idx] = -9.8
	sr.Mass[idx] = mass
	return idx
}

func (sr *SpatialRegistry) RemoveEntity(idx int) {
	sr.mu.Lock()
	defer sr.mu.Unlock()
	if idx >= 0 && idx < len(sr.Active) && sr.Active[idx] {
		sr.Active[idx] = false
		sr.FreeList = append(sr.FreeList, idx)
	}
}

type PotentialFieldRegistry struct {
	mu        sync.RWMutex
	Active    []bool
	FreeList  []int
	Type      []EmitterType
	X         []float32
	Y         []float32
	Z         []float32
	Amplitude []float32
	Sigma     []float32
}

func NewPotentialFieldRegistry() *PotentialFieldRegistry {
	return &PotentialFieldRegistry{
		Active:    make([]bool, 0),
		FreeList:  make([]int, 0),
		Type:      make([]EmitterType, 0),
		X:         make([]float32, 0),
		Y:         make([]float32, 0),
		Z:         make([]float32, 0),
		Amplitude: make([]float32, 0),
		Sigma:     make([]float32, 0),
	}
}

func (pfr *PotentialFieldRegistry) AddEmitter(typ EmitterType, x, y, z, amplitude, sigma float32) int {
	pfr.mu.Lock()
	defer pfr.mu.Unlock()

	idx := -1
	if len(pfr.FreeList) > 0 {
		idx = pfr.FreeList[len(pfr.FreeList)-1]
		pfr.FreeList = pfr.FreeList[:len(pfr.FreeList)-1]
	} else {
		idx = len(pfr.X)
		pfr.Active = append(pfr.Active, false)
		pfr.Type = append(pfr.Type, EmitterGaussian)
		pfr.X = append(pfr.X, 0)
		pfr.Y = append(pfr.Y, 0)
		pfr.Z = append(pfr.Z, 0)
		pfr.Amplitude = append(pfr.Amplitude, 0)
		pfr.Sigma = append(pfr.Sigma, 0)
	}

	pfr.Active[idx] = true
	pfr.Type[idx] = typ
	pfr.X[idx] = x
	pfr.Y[idx] = y
	pfr.Z[idx] = z
	pfr.Amplitude[idx] = amplitude
	pfr.Sigma[idx] = sigma
	return idx
}

func (pfr *PotentialFieldRegistry) RemoveEmitter(idx int) {
	pfr.mu.Lock()
	defer pfr.mu.Unlock()
	if idx >= 0 && idx < len(pfr.Active) && pfr.Active[idx] {
		pfr.Active[idx] = false
		pfr.FreeList = append(pfr.FreeList, idx)
	}
}
