FROM python:3.10-slim-buster as builder
COPY requirements.txt requirements.txt

RUN set -ex && pip install --upgrade pip
RUN set -ex && pip install -r requirements.txt

FROM builder as final
WORKDIR /app
COPY ./app /app/app/
COPY .env /app/
