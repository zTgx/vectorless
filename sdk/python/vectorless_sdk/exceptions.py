"""Exceptions for the vectorless SDK."""


class VectorlessError(Exception):
    """Base exception for all vectorless SDK errors."""

    pass


class ApiError(VectorlessError):
    """API returned an error response."""

    pass


class DocumentNotFound(VectorlessError):
    """Document not found."""

    pass


class InvalidInput(VectorlessError):
    """Invalid input parameter."""

    pass


class AuthenticationFailed(VectorlessError):
    """Authentication failed."""

    pass


class ServiceUnavailable(VectorlessError):
    """Service unavailable."""

    pass


class HttpError(VectorlessError):
    """HTTP request failed."""

    pass
