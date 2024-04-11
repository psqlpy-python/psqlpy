FROM python:3.12.2-slim-bullseye as compile-image

WORKDIR /srv/www/

COPY pyproject* ./
COPY poetry.lock ./

RUN pip install poetry
RUN poetry config virtualenvs.create false
RUN poetry install --no-root

RUN pip uninstall pipenv poetry -y

COPY . .

FROM scratch AS runtime-image

ENV PYTHON_VERSION=3.12.2 \
    PYTHONUNBUFFERED=1

WORKDIR /srv/www/

COPY --from=compile-image / /
