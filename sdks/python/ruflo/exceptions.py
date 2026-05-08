"""Exceptions for Ruflo SDK."""


class RufloError(Exception):
    """Base exception for Ruflo SDK."""
    pass


class RufloAPIError(RufloError):
    """API error from Ruflo."""
    
    def __init__(
        self,
        message: str,
        status_code: Optional[int] = None,
        code: Optional[str] = None,
    ):
        super().__init__(message)
        self.status_code = status_code
        self.code = code


class RufloAuthError(RufloError):
    """Authentication error."""
    pass


class RufloNotFoundError(RufloError):
    """Resource not found error."""
    pass


class RufloRateLimitError(RufloError):
    """Rate limit exceeded error."""
    pass
