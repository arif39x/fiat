![Uclid](asset/uclidlogo.png)

![Go](https://img.shields.io/badge/Go-1.21+-00ADD8?logo=go)
![Rust](https://img.shields.io/badge/Rust-1.75+-DEA584?logo=rust)
![Python](https://img.shields.io/badge/Python-3.11+-3776AB?logo=python)
![WebGPU](https://img.shields.io/badge/WebGPU-WGSL-FF6B35)
![License](https://img.shields.io/badge/license-MIT-green)
![Status](https://img.shields.io/badge/status-alpha-orange)

# Uclid

A math-physics-chemistry compiler..Yes because i am working on it on that time when my mind is mostly weird..

In Uclid, there are no polygons or pre-built models. Everything—the ground you walk on, the shapes you see, and the forces that move you—is defined by mathematical equations you type in real time.

## Quick Start

1. **Install the basics**: You'll need Go, Rust, and Python 3.11+.
2. **Setup**:
   ```bash
   cd compiler && python3 -m venv venv && source venv/bin/activate && pip install fastapi uvicorn pytest
   ```
   ```
   cd ..
   ```
3. **Run**:
   ```bash
   ./start_universe.sh
   ```

## How it works

- **Math (Structure)**: You write Signed Distance Fields (SDFs) to build the world.
- **Physics (Motion)**: The universe uses Gaussian Potential Fields to calculate gravity and movement.
- **Client**: A Rust/WebGPU renderer draws the equations as you type them.
- **Server**: A Go hub keeps everyone in the same reality.
