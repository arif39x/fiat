package main

import (
	"encoding/json"
	"math"
	"time"
)

const CellSize = 10.0 // Grid cell size for spatial partitioning

type GridCoord struct {
	x, y, z int
}

func getGridCoord(x, y, z float32) GridCoord {
	return GridCoord{
		x: int(math.Floor(float64(x / CellSize))),
		y: int(math.Floor(float64(y / CellSize))),
		z: int(math.Floor(float64(z / CellSize))),
	}
}

func StartPhysicsLoop(registry *SpatialRegistry, fields *PotentialFieldRegistry, hub *Hub) {
	ticker := time.NewTicker(time.Millisecond * 50)
	defer ticker.Stop()

	dt := float32(0.05)

	for {
		<-ticker.C

		registry.mu.Lock()
		fields.mu.RLock()

		// Build Spatial Grid for Emitters
		grid := make(map[GridCoord][]int)
		for j := 0; j < len(fields.Active); j++ {
			if !fields.Active[j] {
				continue
			}

			// Determine radius of influence (approx 3 * sigma)
			var radius float32 = 50.0 // Default for Void Zone or other non-gaussian
			if fields.Type[j] == EmitterGaussian {
				radius = 3.0 * fields.Sigma[j]
			}

			minCoord := getGridCoord(fields.X[j]-radius, fields.Y[j]-radius, fields.Z[j]-radius)
			maxCoord := getGridCoord(fields.X[j]+radius, fields.Y[j]+radius, fields.Z[j]+radius)

			for gx := minCoord.x; gx <= maxCoord.x; gx++ {
				for gy := minCoord.y; gy <= maxCoord.y; gy++ {
					for gz := minCoord.z; gz <= maxCoord.z; gz++ {
						coord := GridCoord{gx, gy, gz}
						grid[coord] = append(grid[coord], j)
					}
				}
			}
		}

		// In-process Physics Calculation
		for i := 0; i < len(registry.Active); i++ {
			if !registry.Active[i] {
				continue
			}

			netAX := float32(0.0)
			netAY := float32(0.0)
			netAZ := float32(-9.8)

			mi := registry.Mass[i]
			if mi == 0 {
				mi = 1.0
			}

			coord := getGridCoord(registry.X[i], registry.Y[i], registry.Z[i])
			nearbyFields := grid[coord]

			for _, j := range nearbyFields {
				dx := registry.X[i] - fields.X[j]
				dy := registry.Y[i] - fields.Y[j]
				dz := registry.Z[i] - fields.Z[j]

				if fields.Type[j] == EmitterVoidZone {
					if dx > -25.0 && dx < 25.0 && dy > -25.0 && dy < 25.0 {
						netAZ += 14.8 // counteract -9.8 and add 5.0
					}
					continue
				}

				distSq := dx*dx + dy*dy + dz*dz
				sigmaSq := fields.Sigma[j] * fields.Sigma[j]

				val := fields.Amplitude[j] * float32(math.Exp(float64(-distSq/(2.0*sigmaSq))))
				invMSigmaSq := 1.0 / (mi * sigmaSq)
				netAX += dx * val * invMSigmaSq
				netAY += dy * val * invMSigmaSq
				netAZ += dz * val * invMSigmaSq
			}

			registry.AX[i] = netAX
			registry.AY[i] = netAY
			registry.AZ[i] = netAZ
		}

		var activeX, activeY, activeZ []float32

		for i := 0; i < len(registry.Active); i++ {
			if !registry.Active[i] {
				continue
			}
			registry.VX[i] += registry.AX[i] * dt
			registry.VY[i] += registry.AY[i] * dt
			registry.VZ[i] += registry.AZ[i] * dt

			registry.X[i] += registry.VX[i] * dt
			registry.Y[i] += registry.VY[i] * dt
			registry.Z[i] += registry.VZ[i] * dt

			if registry.Z[i] < -50.0 {
				registry.Z[i] = -50.0
				registry.VZ[i] *= -0.5
			}

			activeX = append(activeX, registry.X[i])
			activeY = append(activeY, registry.Y[i])
			activeZ = append(activeZ, registry.Z[i])
		}

		state := map[string]interface{}{
			"x": activeX,
			"y": activeY,
			"z": activeZ,
		}

		fields.mu.RUnlock()
		registry.mu.Unlock()

		payload, err := json.Marshal(state)
		if err == nil {
			hub.broadcast <- payload
		}
	}
}
