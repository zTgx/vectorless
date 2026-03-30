"""Vectorless Python SDK.

Python SDK for the vectorless service HTTP API.
"""

__version__ = "0.1.0"

from .client import Client, ClientConfig, AsyncClient
from .types import (
    HealthResponse,
    Document,
    CreateDocumentResponse,
    QueryRequest,
    QueryResponse,
    Source,
    UploadContentRequest,
    UploadContentResponse,
)
from .exceptions import (
    VectorlessError,
    ApiError,
    DocumentNotFound,
    InvalidInput,
    AuthenticationFailed,
    ServiceUnavailable,
)

__all__ = [
    "Client",
    "ClientConfig",
    "AsyncClient",
    "HealthResponse",
    "Document",
    "CreateDocumentResponse",
    "QueryRequest",
    "QueryResponse",
    "Source",
    "UploadContentRequest",
    "UploadContentResponse",
    "VectorlessError",
    "ApiError",
    "DocumentNotFound",
    "InvalidInput",
    "AuthenticationFailed",
    "ServiceUnavailable",
    "__version__",
]
