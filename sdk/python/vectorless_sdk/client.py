"""HTTP client for the vectorless service."""

import uuid
from typing import Self

import httpx
from pydantic import BaseModel, Field

from .types import *
from .exceptions import *


class ClientConfig(BaseModel):
    """Client configuration."""

    base_url: str = "http://localhost:8080"
    api_key: str | None = None
    timeout: float = 30.0
    user_agent: str = "vectorless-sdk-py/0.1.0"


class Client:
    """HTTP client for the vectorless service.

    Example:
        ```python
        from vectorless_sdk import Client, ClientConfig

        config = ClientConfig(
            base_url="http://localhost:8080",
            api_key="your-api-key"
        )

        client = Client(config)

        # Check health
        health = await client.health()
        print(f"Service status: {health.status}")

        # Create a document
        doc = await client.create_document("My Document")
        print(f"Created document: {doc.id}")

        # Query the RAG system
        response = await client.query("What is this about?")
        print(f"Answer: {response.answer}")
        ```
    """

    def __init__(self, config: ClientConfig) -> None:
        """Initialize the client.

        Args:
            config: Client configuration
        """
        self.config = config
        self._client = httpx.Client(
            base_url=config.base_url,
            timeout=config.timeout,
            headers={
                "Content-Type": "application/json",
                "User-Agent": config.user_agent,
                **({"X-API-Key": config.api_key} if config.api_key else {}),
            },
        )

    @classmethod
    def default(cls) -> Self:
        """Create a client with default configuration.

        Returns:
            Client instance with default configuration
        """
        return cls(ClientConfig())

    def _handle_response(self, response: httpx.Response) -> None:
        """Handle API response and raise appropriate errors.

        Args:
            response: HTTP response

        Raises:
            AuthenticationFailed: If authentication fails (401)
            ServiceUnavailable: If service is unavailable (503)
            ApiError: If API returns an error response
        """
        if response.status_code == 401:
            raise AuthenticationFailed("Authentication failed")
        if response.status_code == 503:
            raise ServiceUnavailable("Service unavailable")
        if response.status_code >= 400:
            try:
                error = response.json()
                error_resp = ApiErrorResponse.model_validate(error)
                raise ApiError(error_resp.error)
            except Exception:
                raise ApiError(f"HTTP {response.status_code}: {response.text}")

    async def health(self) -> HealthResponse:
        """Check service health.

        Returns:
            Health check response

        Raises:
            HttpError: If HTTP request fails
        """
        response = self._client.get("/health")
        response.raise_for_status()
        return HealthResponse.model_validate_json(response.content)

    async def list_documents(self) -> list[Document]:
        """List all documents.

        Returns:
            List of documents

        Raises:
            HttpError: If HTTP request fails
            ApiError: If API returns an error
        """
        response = self._client.get("/documents")
        self._handle_response(response)
        response.raise_for_status()
        data = response.json()
        return [Document.model_validate(item) for item in data]

    async def create_document(self, title: str) -> CreateDocumentResponse:
        """Create a new document.

        Args:
            title: Document title

        Returns:
            Create document response

        Raises:
            HttpError: If HTTP request fails
            ApiError: If API returns an error
        """
        request = CreateDocumentRequest(title=title)
        response = self._client.post("/documents", content=request.model_dump_json())
        self._handle_response(response)
        response.raise_for_status()
        return CreateDocumentResponse.model_validate_json(response.content)

    async def get_document(self, id: uuid.UUID) -> Document:
        """Get document by ID.

        Args:
            id: Document ID

        Returns:
            Document metadata

        Raises:
            HttpError: If HTTP request fails
            ApiError: If API returns an error
            DocumentNotFound: If document doesn't exist
        """
        response = self._client.get(f"/documents/{id}")
        self._handle_response(response)
        if response.status_code == 404:
            raise DocumentNotFound(f"Document not found: {id}")
        response.raise_for_status()
        return Document.model_validate_json(response.content)

    async def delete_document(self, id: uuid.UUID) -> DeleteDocumentResponse:
        """Delete a document.

        Args:
            id: Document ID

        Returns:
            Delete document response

        Raises:
            HttpError: If HTTP request fails
            ApiError: If API returns an error
        """
        response = self._client.delete(f"/documents/{id}")
        self._handle_response(response)
        response.raise_for_status()
        return DeleteDocumentResponse.model_validate_json(response.content)

    async def upload_content(self, id: uuid.UUID, content: str) -> UploadContentResponse:
        """Upload document content.

        Args:
            id: Document ID
            content: Document content

        Returns:
            Upload content response

        Raises:
            HttpError: If HTTP request fails
            ApiError: If API returns an error
        """
        request = UploadContentRequest(content=content)
        response = self._client.post(
            f"/documents/{id}/content",
            content=request.model_dump_json()
        )
        self._handle_response(response)
        response.raise_for_status()
        return UploadContentResponse.model_validate_json(response.content)

    async def query(self, query: str) -> QueryResponse:
        """Query the RAG system.

        Args:
            query: Query text

        Returns:
            Query response with answer and sources

        Raises:
            HttpError: If HTTP request fails
            ApiError: If API returns an error
        """
        request = QueryRequest(query=query)
        response = self._client.post("/query", content=request.model_dump_json())
        self._handle_response(response)
        response.raise_for_status()
        return QueryResponse.model_validate_json(response.content)


# Async context manager support
class AsyncClient(Client):
    """Async client with context manager support.

    Example:
        ```python
        async with AsyncClient.default() as client:
            health = await client.health()
            print(f"Status: {health.status}")
        ```
    """

    async def __aenter__(self) -> Self:
        return self

    async def __aexit__(self, *args) -> None:
        await self.aclose()

    async def aclose(self) -> None:
        """Close the HTTP client."""
        await self._client.aclose()
