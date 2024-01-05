from app.core.log_config import init_loggers
from github import Github
from github import Auth
from github.GithubException import BadCredentialsException


loggerIH = init_loggers(__name__)


class GitHubClient:
    def __init__(self,
                 access_token,
                 HOSTNAME_GH,
                 organization_name):
        try:
            auth = Auth.Token(access_token)
            loggerIH.info(f"Auth built")
            self.g = Github(base_url=f"https://{HOSTNAME_GH}/api/v3", auth=auth)
            user = self.g.get_user()
            loggerIH.info(f"Authenticated as {user.login}")
            self.user = user
            self.org = self.g.get_organization(organization_name)
            loggerIH.info(f"Organization {self.org.login} fetched successfully")
        except BadCredentialsException:
            loggerIH.fatal("Authentication failed. Please check your access token.")
            raise BadCredentialsException

    def get_repo(self,
                 repository_name):
        return self.org.get_repo(repository_name)

    def get_contents(self,
                     repository_name,
                     folder_path):
        repo = self.get_repo(repository_name)
        return repo.get_contents(folder_path)
