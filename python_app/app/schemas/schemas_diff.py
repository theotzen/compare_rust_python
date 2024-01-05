from datetime import datetime
from typing import Optional, List

from pydantic import BaseModel, Field
from bson import ObjectId


class DiffBaseSchema(BaseModel):
    stackA: str
    stackB: str
    file: str
    leftNotRight: List[str]
    rightNotLeft: List[str]
    sameKeyDiffValue: List[str]
    created_at: Optional[datetime] = datetime.utcnow()
    updated_at: Optional[datetime] = None

    class Config:
        orm_mode = True


class PyObjectId(str):
    @classmethod
    def __get_validators__(cls):
        yield cls.validate

    @classmethod
    def validate(cls, v, field):
        if not isinstance(v, ObjectId):
            raise TypeError('ObjectId required')
        return str(v)


class FileDiff(BaseModel):
    id: Optional[PyObjectId] = Field(alias='_id')
    stackA: str
    stackB: str
    file: str
    leftNotRight: List[str]
    rightNotLeft: List[str]
    sameKeyDiffValue: List[str]
    reviewed: Optional[str]

    class Config:
        json_encoders = {
            ObjectId: str
        }
        allow_population_by_field_name = True


class GetOneDiffResponse(BaseModel):
    diff: FileDiff


class GetAllDiffsResponse(BaseModel):
    stackA: str
    stackB: str
    filesWithDiff: List[FileDiff]


class GetAllDiffsNoStackResponse(BaseModel):
    filesWithDiff: List[FileDiff]


class GetAllDiffsPayload(BaseModel):
    stackA: str
    stackB: str


class GetLatestDiffResponse(BaseModel):
    stackA: str
    stackB: str
    diff: FileDiff
    latestDiff: str


class GetConfigsFromStacksPayload(BaseModel):
    stackA: str
    stackB: str
    file: str


class GetConfigsFromStacksResponse(BaseModel):
    stackA: str
    stackB: str
    file: str
    configA: str
    configB: str


class ComputeAllDiffPayload(BaseModel):
    stackA: str
    stackB: str


class ComputeAllDiffResponse(BaseModel):
    stackA: str
    stackB: str
    filesWithDiff: List[FileDiff]


class ToggleReviewPayload(BaseModel):
    id: str


class ToggleReviewResponse(BaseModel):
    status: str
