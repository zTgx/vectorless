"""Data types for the vectorless service API."""

from uuid import UUID
from pydantic import BaseModel, Field


class HealthResponse(BaseModel):
    """Health check response."""

    status: str


class CreateDocumentRequest(BaseModel):
    """Create document request."""

    title: str


class CreateDocumentResponse(BaseModel):
    """Create document response."""

    id: UUID
    status: str


class Document(BaseModel):
    """Document metadata."""

    id: UUID
    doc_type: str = Field(alias="type")
    title: str
    doc_description: str
    status: str
    page_count: int | None = None
    line_count: int | None = None
    created_at: str | None = None
    modified_at: str | None = None

    class Config:
        populate_by_name = True


class UploadContentRequest(BaseModel):
    """Upload document content request."""

    content: str


class UploadContentResponse(BaseModel):
    """Upload content response."""

    message: str
    bytes: int


class QueryRequest(BaseModel):
    """Query request for RAG."""

    query: str


class Source(BaseModel):
    """Source reference in RAG response."""

    document_id: UUID
    section: str
    content: str


class QueryResponse(BaseModel):
    """Query response from RAG system."""

    answer: str
    sources: list[Source]


class DeleteDocumentResponse(BaseModel):
    """Delete document response."""

    message: str


class ApiErrorResponse(BaseModel):
    """API error response."""

    error: str
