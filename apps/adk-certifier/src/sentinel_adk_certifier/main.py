"""Main entry point for the ADK Certifier service."""

import uvicorn

from sentinel_adk_certifier.config import Settings
from sentinel_adk_certifier.server import create_app


def main() -> None:
    settings = Settings()
    app = create_app(settings)
    uvicorn.run(
        app,
        host=settings.host,
        port=settings.port,
        log_level=settings.log_level.lower(),
    )


if __name__ == "__main__":
    main()
