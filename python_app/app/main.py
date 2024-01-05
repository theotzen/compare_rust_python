from app.routers import diff
from app.config import settings

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=[settings.ORIGINS],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(diff.router, tags=["Diff"], prefix="/api/diff")


@app.get("/simple_json")
async def simple_json():
    return {"message": "Rust vs Python"}
