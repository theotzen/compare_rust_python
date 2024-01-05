from app.core.log_config import init_loggers
from fastapi import Depends
from github.GithubException import BadCredentialsException

from app.github.github_client import GitHubClient


from app.config import settings

loggerIH = init_loggers(__name__)

github_client_instance = None


def check_connection(client: GitHubClient) -> bool:
    try:
        client.g.get_user()
        return True
    except BadCredentialsException:
        return False


def get_github_client() -> GitHubClient:
    global github_client_instance
    if github_client_instance is None or not check_connection(github_client_instance):
        github_client_instance = GitHubClient(settings.ACCESS_TOKEN_GH, settings.HOSTNAME_GH, settings.ORGANIZATION_GH)
    return github_client_instance


async def github_client_dependency() -> GitHubClient:
    return get_github_client()
