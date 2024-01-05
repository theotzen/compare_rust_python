import pymongo
from app.config import settings
from app.core.log_config import init_loggers
from pymongo import mongo_client

loggerIH = init_loggers(__name__)

client = mongo_client.MongoClient(
    settings.DATABASE_URL, serverSelectionTimeoutMS=5000)

try:
    conn = client.server_info()
    loggerIH.info(f'Connected to MongoDB {conn.get("version")}')
except (Exception,):
    loggerIH.exception(
        "Unable to connect to the MongoDB server.", Exception.args)

db = client[settings.MONGO_INITDB_DATABASE]
User = db.users
User.create_index([("email", pymongo.ASCENDING)], unique=True)
Diff = db.diffs
