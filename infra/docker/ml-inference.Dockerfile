FROM python:3.12-slim

ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1
ENV UV_PROJECT_ENVIRONMENT=/opt/oddex-ml-venv

RUN pip install --no-cache-dir uv

WORKDIR /app/ml

COPY ml/pyproject.toml ml/uv.lock ./
COPY ml/serve_xgboost.py ./
COPY ml/serve_trainer.py ./
COPY ml/refresh_model_bundle.py ./
COPY ml/train_xgboost.py ./
COPY ml/xgboost_pipeline.py ./

RUN uv sync --project /app/ml --locked

EXPOSE 8010

CMD ["sh", "-lc", "while [ ! -f \"${ML_MODEL_DIR:-/models/btc-15m-xgb-policy}/model.json\" ] || [ ! -f \"${ML_MODEL_DIR:-/models/btc-15m-xgb-policy}/metadata.json\" ]; do echo \"waiting for model bundle in ${ML_MODEL_DIR:-/models/btc-15m-xgb-policy}\"; sleep 5; done; exec uv run --project /app/ml python /app/ml/serve_xgboost.py --model-dir \"${ML_MODEL_DIR:-/models/btc-15m-xgb-policy}\" --host \"${ML_HOST:-0.0.0.0}\" --port \"${ML_PORT:-8010}\""]
