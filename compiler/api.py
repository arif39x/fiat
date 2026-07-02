from typing import Any, Dict

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

from core.jobs import JobQueue
from core.ml.model_registry import get_default_registry
from core.animation.retarget import retarget_clip, build_bone_name_map
from core.animation.motion import MotionClip
from core.animation.skeleton import Skeleton

app = FastAPI()


class JobRequest(BaseModel):
    job_type: str
    params: Dict[str, Any]


job_queue = JobQueue()
model_registry = get_default_registry()


@app.post("/jobs")
async def create_job(req: JobRequest):
    job_id = await job_queue.enqueue(req.job_type, req.params)
    return {"job_id": job_id, "status": "queued"}


@app.get("/jobs/{job_id}")
async def get_job(job_id: str):
    job = job_queue.get_job(job_id)
    if job is None:
        raise HTTPException(status_code=404, detail="Job not found")
    return {
        "id": job.id,
        "job_type": job.job_type,
        "status": job.status.value,
        "progress": job.progress,
        "error": job.error,
        "result": job.result,
    }


@app.post("/generate_character")
async def generate_character(params: Dict[str, Any]):
    _use, _endpoint = model_registry.resolve("text_to_mesh")
    return {
        "status": "success",
        "mesh": {"vertices": [], "indices": []},
        "skeleton": {"name": "default", "joints": []},
        "clip": {"frames": [], "fps": 30, "loop": True},
        "fallback_mode": _use == "fallback",
    }


@app.post("/generate_motion")
async def generate_motion(params: Dict[str, Any]):
    _use, _endpoint = model_registry.resolve("text_to_motion")
    return {
        "status": "success",
        "clip": {"frames": [], "fps": 30, "loop": True},
        "fallback_mode": _use == "fallback",
    }


@app.post("/stage_pose")
async def stage_pose(params: Dict[str, Any]):
    _use, _endpoint = model_registry.resolve("pose_interpolation")
    return {
        "status": "success",
        "clip": {"frames": [], "fps": 30, "loop": False},
        "fallback_mode": _use == "fallback",
    }


@app.post("/style_transfer")
async def style_transfer(params: Dict[str, Any]):
    _use, _endpoint = model_registry.resolve("style_transfer")
    return {
        "status": "success",
        "clip": {"frames": [], "fps": 30, "loop": False},
        "fallback_mode": _use == "fallback",
    }


@app.post("/retarget")
async def retarget_animation(params: Dict[str, Any]):
    return {
        "status": "success",
        "clip": {"frames": [], "fps": 30, "loop": False},
    }


@app.post("/export")
async def export_asset(params: Dict[str, Any]):
    fmt = params.get("format", "glb")
    return {
        "status": "success",
        "format": fmt,
        "data": None,
        "message": f"Export to {fmt} requested",
    }
