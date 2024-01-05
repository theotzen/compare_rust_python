import datetime

from app.core.log_config import init_loggers
from app.database import Diff

from bson import ObjectId
from fastapi import APIRouter, status, Depends, HTTPException
from github.GithubException import UnknownObjectException

from app.config import settings
from app.github.github_client import GitHubClient
from app.github.github_client_dependency import github_client_dependency
from app.serializers.diff_serializers import allDiffsResponseEntity
from app.schemas import schemas_diff
from app.utils import get_config_from_gh, handle_rate_limit_async, compare_yaml_strings, check_contents_and_take_first


loggerIH = init_loggers(__name__)

router = APIRouter()


@router.get(path="/getOneDiffById/{diff_id}",
            status_code=status.HTTP_200_OK,
            response_model=schemas_diff.GetOneDiffResponse)
async def get_diff_by_id(diff_id: str):
    diff = Diff.find_one({"_id": ObjectId(diff_id)})
    if not diff:
        loggerIH.error(
            f"{status.HTTP_400_BAD_REQUEST} | No diff found")
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST,
                            detail="No diff found with this Id")
    res = {
        "diff": allDiffsResponseEntity(diff)
    }
    return res


@router.get(path="/getAllDiff",
            status_code=status.HTTP_200_OK,
            response_model=schemas_diff.GetAllDiffsNoStackResponse)
async def get_all_diff():

    all_diffs = Diff.find({})
    loggerIH.info("Got all diffs")

    diffs = list(all_diffs)
    loggerIH.info("Got all diffs as list")

    loggerIH.info((diffs[0]))

    if not diffs:
        loggerIH.error(
            f"{status.HTTP_400_BAD_REQUEST} | No diff found")
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST,
                            detail="No comparison found for these stacks")
    res = {
        "filesWithDiff": [allDiffsResponseEntity(diff) for diff in diffs]
    }
    return res


@router.post(path="/getLatestDiffsFromStacks",
             status_code=status.HTTP_200_OK,
             response_model=schemas_diff.GetLatestDiffResponse)
async def get_latest_diffs_from_stacks(payload: schemas_diff.GetConfigsFromStacksPayload):

    diffs = list(Diff.find({"stackA": payload.stackA, "stackB": payload.stackB, "file": payload.file}))

    if not diffs:
        loggerIH.error(
            f"{status.HTTP_400_BAD_REQUEST} | No comparison found with {payload.stackA} and {payload.stackB}")
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST,
                            detail="No comparison found for these stacks")

    set_creation_time = set()

    for diff in diffs:
        set_creation_time.add(diff["created_at"])
    time_to_keep = max(set_creation_time)

    diffs = [diff for diff in diffs if diff["created_at"] == time_to_keep]

    if diffs:
        diff_to_keep = diffs[0]

    res = {
        "stackA": payload.stackA, "stackB": payload.stackB,
        "diff": diff_to_keep,
        "latestDiff": str(time_to_keep)
    }
    return res


@router.post(path="/getAllDiffsFromStacks",
             status_code=status.HTTP_200_OK,
             response_model=schemas_diff.GetAllDiffsResponse)
async def get_all_diffs_from_stacks(payload: schemas_diff.GetAllDiffsPayload):

    diffs = list(Diff.find({"stackA": payload.stackA, "stackB": payload.stackB}))

    if not diffs:
        loggerIH.error(
            f"{status.HTTP_400_BAD_REQUEST} | No comparison found with {payload.stackA} and {payload.stackB}")
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST,
                            detail="No comparison found for these stacks")

    set_creation_time = set()

    for diff in diffs:
        set_creation_time.add(diff["created_at"])
    time_to_keep = max(set_creation_time)

    diffs = [diff for diff in diffs if diff["created_at"] == time_to_keep]
    res = {
        "stackA": payload.stackA, "stackB": payload.stackB,
        "filesWithDiff": [allDiffsResponseEntity(diff) for diff in diffs]
    }
    return res


@router.post(path="/getConfigsFromStacks",
             status_code=status.HTTP_200_OK,
             response_model=schemas_diff.GetConfigsFromStacksResponse)
async def get_configs_from_stacks_name(payload: schemas_diff.GetConfigsFromStacksPayload,
                                  g: GitHubClient = Depends(github_client_dependency)):
    configA = await get_config_from_gh(g, payload.stackA, payload.file)
    configB = await get_config_from_gh(g, payload.stackB, payload.file)
    res = payload.dict()
    res["configA"] = configA
    res["configB"] = configB
    return res


@router.post(path="/computeAllDiffs",
             status_code=status.HTTP_200_OK,
             response_model=schemas_diff.ComputeAllDiffResponse)
async def compute_diff_for_all_files(payload: schemas_diff.ComputeAllDiffPayload,
                                       g: GitHubClient = Depends(github_client_dependency)):
    await handle_rate_limit_async(g.g)

    now = datetime.datetime.now()
    filesWithDiff = []

    try:
        repo_A = g.org.get_repo(payload.stackA)
        repo_B = g.org.get_repo(payload.stackB)
    except UnknownObjectException as err:
        loggerIH.error(f"UnknownObjectException occurred while getting repo: {str(err)}")
        raise UnknownObjectException("Could not find repo")

    # Getting all services we want to compare config of
    try:
        folder_contents_A = repo_A.get_contents(settings.FOLDER_PATH + settings.FOLDER_A_NAME)
        subfolders_A = [content.path for content in folder_contents_A if content.type == "dir"]

        loggerIH.info(f"Got {len(subfolders_A)} services in {settings.FOLDER_A_NAME}")

        folder_contents_B = repo_A.get_contents(settings.FOLDER_PATH + settings.FOLDER_B_NAME)
        subfolders_B = [content.path for content in folder_contents_B if content.type == "dir"]

        loggerIH.info(f"Got {len(subfolders_B)} services in {settings.FOLDER_B_NAME}")
    except UnknownObjectException as err:
        loggerIH.error(f"UnknownObjectException occurred while getting subfolders: {str(err)}")
        raise UnknownObjectException("Could not find subfolders")

    subfolders = subfolders_A + subfolders_B
    for path in subfolders:
        res = payload.dict()
        res["created_at"] = now
        res["updated_at"] = now
        res["file"] = path
        res["stackA"] = payload.stackA
        res["stackB"] = payload.stackB
        res["reviewed"] = "false"
        file_path = path + "/config-overrides.yml"

        # Getting file content for stack_A
        try:
            file_contents_A = repo_A.get_contents(file_path)
            file_contents_A = check_contents_and_take_first(file_contents_A)
            file_content_str_A = file_contents_A.decoded_content.decode('utf-8')
            loggerIH.info(f"Got contents for : {payload.stackA}/{file_path}")
            loggerIH.info(f"Contents is of size {len(file_content_str_A)}")
        except UnknownObjectException as err:
            loggerIH.error(f"UnknownObjectException occurred while getting contents: {str(err)}")
            raise UnknownObjectException("Could not find contents")

        # Getting file content for stack_B. If the file doesn't exist, we insert a diff and continue
        try:
            file_contents_B = repo_B.get_contents(file_path)
            file_contents_B = check_contents_and_take_first(file_contents_B)
            file_content_str_B = file_contents_B.decoded_content.decode('utf-8')
            loggerIH.info(f"Got contents for : {payload.stackB}/{file_path}")
            loggerIH.info(f"Contents is of size {len(file_content_str_B)}")
        except UnknownObjectException as err:
            loggerIH.warn(f"UnknownObjectException occurred while getting contents for stackB: {str(err)}")

            res["leftNotRight"] = ["/*"]
            res["rightNotLeft"] = []
            res["sameKeyDiffValue"] = []

            mongo_result = Diff.insert_one(res)

            loggerIH.info(f"Tried to insert in DB : {mongo_result.inserted_id}")
            loggerIH.info(f"Got result {mongo_result}")

            filesWithDiff.append(
                schemas_diff.FileDiff(
                    _id=mongo_result.inserted_id,
                    stackA=res["stackA"],
                    stackB=res["stackB"],
                    file=res["file"],
                    leftNotRight=res["leftNotRight"],
                    rightNotLeft=res["rightNotLeft"],
                    sameKeyDiffValue=res["sameKeyDiffValue"],
                    reviewed=res["reviewed"]
                )
            )
            continue

        # If we got contents for both stacks, we can compare them
        leftNotRight, rightNotLeft, sameKeySameValue, sameKeyDiffValue = compare_yaml_strings(file_content_str_A,
                                                                                              file_content_str_B)

        if not leftNotRight and not rightNotLeft and not sameKeyDiffValue and sameKeySameValue:
            loggerIH.info(f"The config {path} is the same for both, not adding to the DB and continuing")
            continue

        loggerIH.info(f"Computed yaml diff of {path}")

        res["leftNotRight"] = leftNotRight
        res["rightNotLeft"] = rightNotLeft
        res["sameKeyDiffValue"] = sameKeyDiffValue

        mongo_result = Diff.insert_one(res)

        loggerIH.info(f"Tried to insert in DB : {mongo_result.inserted_id}")
        loggerIH.info(f"Got result {mongo_result}")

        filesWithDiff.append(
            schemas_diff.FileDiff(
                _id=mongo_result.inserted_id,
                stackA=res["stackA"],
                stackB=res["stackB"],
                file=res["file"],
                leftNotRight=res["leftNotRight"],
                rightNotLeft=res["rightNotLeft"],
                sameKeyDiffValue=res["sameKeyDiffValue"],
                reviewed=res["reviewed"]
            )
        )
    response = payload.dict()
    response["filesWithDiff"] = filesWithDiff
    return response


@router.post(path="/toggleReview",
             status_code=status.HTTP_200_OK,
             response_model=schemas_diff.ToggleReviewResponse)
async def toggle_review(payload: schemas_diff.ToggleReviewPayload):

    loggerIH.info(f"Toggling review for {payload.id}")

    diff = Diff.find_one({"_id": ObjectId(payload.id)})

    if not diff:
        loggerIH.error(
            f"{status.HTTP_400_BAD_REQUEST} | No diff found")
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST,
                            detail="No diff found with this Id")

    diff = allDiffsResponseEntity(diff)
    if diff["reviewed"] == "false":
        new_reviewed = "true"
    else:
        new_reviewed = "false"

    result = Diff.update_one(
        {"_id": ObjectId(payload.id)},
        {"$set": {'reviewed': new_reviewed}}
    )

    res = {
        "status": str(result.acknowledged)
    }
    return res