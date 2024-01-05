import os
import logging
import uvicorn

LOG_LEVEL: str = "DEBUG"
FORMAT: str = "%(levelprefix)s %(message)s | %(asctime)s"


def init_loggers(logger_name: str = "default_logger"):
    logger = logging.getLogger(logger_name)
    logger.setLevel(logging.DEBUG)

    formatter = uvicorn.logging.DefaultFormatter(
        FORMAT, datefmt="%Y-%m-%d %H:%M:%S")

    ch = logging.StreamHandler()
    ch.setLevel(logging.DEBUG)
    ch.setFormatter(formatter)

    log_directory = "app/logs"
    if not os.path.exists(log_directory):
        os.makedirs(log_directory)

    fh = logging.FileHandler(f"app/logs/last_logs_{logger_name}.log")
    fh.setFormatter(formatter)
    fh.setLevel(logging.INFO)

    logger.addHandler(ch)
    logger.addHandler(fh)

    return logger
