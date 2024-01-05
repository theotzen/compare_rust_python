from app.core.log_config import init_loggers
from app.github.github_client import GitHubClient

import asyncio
from github import Github
from github.GithubException import UnknownObjectException
import time
import yaml


loggerIH = init_loggers(__name__)


async def handle_rate_limit_async(g: Github):
    rate_limit = g.get_rate_limit()
    remaining = rate_limit.core.remaining

    loggerIH.info(f"rate_limit :  {rate_limit}")
    loggerIH.info(f"remaining :  {remaining}")

    if remaining == 0:
        reset_timestamp = rate_limit.core.reset
        sleep_time = reset_timestamp - time.time()
        if sleep_time > 0:
            await asyncio.sleep(sleep_time)


def check_contents_and_take_first(file_contents):
    if isinstance(file_contents, list):
        if not file_contents:
            raise ValueError
        loggerIH.info(f"Got several ContentFile : {', '.join([content.path for content in file_contents])}")
        loggerIH.info(f"Taking first one : {file_contents[0].path}")
        file_contents = file_contents[0]

    return file_contents


async def get_config_from_gh(g: GitHubClient, stack: str, file: str):
    await handle_rate_limit_async(g.g)
    try:
        repo = g.org.get_repo(stack)
    except UnknownObjectException as err:
        loggerIH.error(f"UnknownObjectException occurred while getting repo: {str(err)}")
        raise UnknownObjectException("Could not find repo")
    try:
        file_contents = repo.get_contents(file)
        file_contents = check_contents_and_take_first(file_contents)
        file_content_str = file_contents.decoded_content.decode('utf-8')
    except UnknownObjectException as err:
        loggerIH.error(f"UnknownObjectException occurred while getting contents: {str(err)}")
        raise UnknownObjectException("Could not find content")

    return file_content_str

            
def compare_dicts(dict_A, dict_B):
    """
    :param dict_A:
    :param dict_B:
    :return: @FileDiff

    This function takes two dictionaries and compare all keys (presence and values). Depth-first search.

    The leftNotRight and rightNotLeft arrays will NOT contain nested keys if a sub dictionary is totally absent
    from the other one. For example, if dict_A contains { c:pyth {a: 1, b:2 }} but dict_B has no "c" key,
    leftNotRight will contain "/c" but not "/c/a" and "/c/b"
    """

    loggerIH.info(f"Comparing two dicts")

    leftNotRight = []
    rightNotLeft = []
    sameKeySameValue = []
    sameKeyDiffValue = []

    stack = [("", dict_A, dict_B)]
    while stack:
        path, current_dict_A, current_dict_B = stack.pop()

        for key in current_dict_A:
            if key not in current_dict_B:
                loggerIH.info(f"{path}/{key} is in A but not in B")
                leftNotRight.append(f"{path}/{key}")
            else:
                if isinstance(current_dict_A[key], dict) and isinstance(current_dict_B[key], dict):
                    if not current_dict_A[key] and not current_dict_B[key]:  # Check if both dictionaries are empty
                        loggerIH.info("Both dictionaries are equally empty")
                        sameKeySameValue.append(f"{path}/{key}")
                    else:
                        loggerIH.info("Dictionaries aren't empty")
                        stack.append((f"{path}/{key}", current_dict_A[key], current_dict_B[key]))
                elif current_dict_A[key] == current_dict_B[key]:
                    loggerIH.info(f"{path}/{key} is in both with the same value")
                    sameKeySameValue.append(f"{path}/{key}")
                else:
                    loggerIH.info(f"{path}/{key} is in both but with different values")
                    sameKeyDiffValue.append(f"{path}/{key}")

        for key in current_dict_B:
            if key not in current_dict_A:
                loggerIH.info(f"{path}/{key} is in B but not in A")
                rightNotLeft.append(f"{path}/{key}")

    return leftNotRight, rightNotLeft, sameKeySameValue, sameKeyDiffValue


def compare_yaml_strings(yaml_A_content, yaml_B_content):
    dict_A = yaml.safe_load(yaml_A_content)
    loggerIH.info(f"Loaded yaml_A as dict with {len(list(dict_A.keys()))} keys")

    dict_B = yaml.safe_load(yaml_B_content)
    loggerIH.info(f"Loaded yaml_A as dict with {len(list(dict_B.keys()))} keys")

    leftNotRight, rightNotLeft, sameKeySameValue, sameKeyDiffValue = compare_dicts(dict_A, dict_B)

    loggerIH.info(f"After comparing yamls, leftNotRight is of size : {len(leftNotRight)}")
    loggerIH.info(f"After comparing yamls, rightNotLeft is of size : {len(rightNotLeft)}")
    loggerIH.info(f"After comparing yamls, sameKeySameValue is of size : {len(sameKeySameValue)}")
    loggerIH.info(f"After comparing yamls, sameKeyDiffValue is of size : {len(sameKeyDiffValue)}")

    return leftNotRight, rightNotLeft, sameKeySameValue, sameKeyDiffValue
