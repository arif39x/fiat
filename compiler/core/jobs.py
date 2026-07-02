from __future__ import annotations

import asyncio
import uuid
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Any, Awaitable, Callable, Dict, Optional


class JobStatus(str, Enum):
    QUEUED = "queued"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"


@dataclass
class Job:
    id: str
    job_type: str
    params: Dict[str, Any]
    status: JobStatus = JobStatus.QUEUED
    progress: float = 0.0
    result: Optional[Any] = None
    error: Optional[str] = None
    created_at: datetime = field(default_factory=datetime.utcnow)
    updated_at: datetime = field(default_factory=datetime.utcnow)


ProgressCallback = Callable[[str, float, Optional[str]], Awaitable[None]]


class JobQueue:
    def __init__(self, progress_callback: Optional[ProgressCallback] = None):
        self._jobs: Dict[str, Job] = {}
        self._queue: asyncio.Queue[str] = asyncio.Queue()
        self._handlers: Dict[str, Callable[[Job], Awaitable[Any]]] = {}
        self._progress_callback = progress_callback

    def register_handler(self, job_type: str, handler: Callable[[Job], Awaitable[Any]]):
        self._handlers[job_type] = handler

    async def enqueue(self, job_type: str, params: Dict[str, Any]) -> str:
        job_id = str(uuid.uuid4())
        job = Job(id=job_id, job_type=job_type, params=params)
        self._jobs[job_id] = job
        await self._queue.put(job_id)
        return job_id

    def get_job(self, job_id: str) -> Optional[Job]:
        return self._jobs.get(job_id)

    def get_all_jobs(self) -> list[Job]:
        return list(self._jobs.values())

    async def update_progress(self, job_id: str, progress: float, message: Optional[str] = None):
        job = self._jobs.get(job_id)
        if job is None:
            return
        job.progress = progress
        job.updated_at = datetime.utcnow()
        if self._progress_callback:
            await self._progress_callback(job_id, progress, message)

    async def complete(self, job_id: str, result: Any):
        job = self._jobs.get(job_id)
        if job is None:
            return
        job.status = JobStatus.COMPLETED
        job.progress = 1.0
        job.result = result
        job.updated_at = datetime.utcnow()
        if self._progress_callback:
            await self._progress_callback(job_id, 1.0, "completed")

    async def fail(self, job_id: str, error: str):
        job = self._jobs.get(job_id)
        if job is None:
            return
        job.status = JobStatus.FAILED
        job.error = error
        job.updated_at = datetime.utcnow()
        if self._progress_callback:
            await self._progress_callback(job_id, 0.0, f"failed: {error}")

    async def process_loop(self):
        while True:
            job_id = await self._queue.get()
            job = self._jobs.get(job_id)
            if job is None:
                continue
            job.status = JobStatus.RUNNING
            handler = self._handlers.get(job.job_type)
            if handler is None:
                await self.fail(job_id, f"No handler registered for job type '{job.job_type}'")
                continue
            try:
                result = await handler(job)
                await self.complete(job_id, result)
            except Exception as e:
                await self.fail(job_id, str(e))
