from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    DATABASE_URL: str
    MONGO_INITDB_DATABASE: str
    MONGO_INITDB_ROOT_USERNAME: str
    MONGO_INITDB_ROOT_PASSWORD: str
    ACCESS_TOKEN_GH: str
    HOSTNAME_GH: str
    ORGANIZATION_GH: str
    FOLDER_PATH: str
    ORIGINS: str
    FOLDER_A_NAME: str
    FOLDER_B_NAME: str

    class Config:
        env_file = './.env'


settings = Settings()
