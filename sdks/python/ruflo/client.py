"""Ruflo API Client."""

from typing import Optional, Dict, Any, List
import requests
import json

from .exceptions import RufloAPIError, RufloAuthError
from .models import Meeting, Transcription, MeetingAnalysis, Workflow


class Client:
    """Main client for interacting with the Ruflo API."""
    
    def __init__(
        self,
        api_key: str,
        api_secret: Optional[str] = None,
        base_url: str = "https://api.ruflo.io/v1",
        timeout: int = 30,
    ):
        """
        Initialize the Ruflo client.
        
        Args:
            api_key: Your Ruflo API key
            api_secret: Your Ruflo API secret (optional)
            base_url: The base URL for the API
            timeout: Request timeout in seconds
        """
        self.api_key = api_key
        self.api_secret = api_secret
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        
        self.session = requests.Session()
        self.session.headers.update({
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        })
        
        # Initialize resource clients
        self.meetings = MeetingsClient(self)
        self.transcriptions = TranscriptionsClient(self)
        self.workflows = WorkflowsClient(self)
        self.predictions = PredictionsClient(self)
        self.translations = TranslationsClient(self)
    
    def _request(
        self,
        method: str,
        path: str,
        data: Optional[Dict[str, Any]] = None,
        params: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """Make an HTTP request to the API."""
        url = f"{self.base_url}{path}"
        
        try:
            response = self.session.request(
                method=method,
                url=url,
                json=data,
                params=params,
                timeout=self.timeout,
            )
            response.raise_for_status()
            return response.json()
        except requests.exceptions.HTTPError as e:
            if response.status_code == 401:
                raise RufloAuthError("Invalid API key")
            elif response.status_code == 403:
                raise RufloAuthError("Insufficient permissions")
            else:
                error_data = response.json() if response.content else {}
                raise RufloAPIError(
                    message=error_data.get("message", str(e)),
                    status_code=response.status_code,
                    code=error_data.get("code"),
                )
        except requests.exceptions.RequestException as e:
            raise RufloAPIError(f"Request failed: {str(e)}")
    
    def _get(self, path: str, params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        return self._request("GET", path, params=params)
    
    def _post(self, path: str, data: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        return self._request("POST", path, data=data)
    
    def _put(self, path: str, data: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        return self._request("PUT", path, data=data)
    
    def _delete(self, path: str) -> None:
        self._request("DELETE", path)


class MeetingsClient:
    """Client for meetings API."""
    
    def __init__(self, client: Client):
        self._client = client
    
    def list(
        self,
        org_id: Optional[str] = None,
        status: Optional[str] = None,
        from_date: Optional[str] = None,
        to_date: Optional[str] = None,
        limit: int = 20,
        offset: int = 0,
    ) -> Dict[str, Any]:
        """List meetings with optional filters."""
        params = {
            "org_id": org_id,
            "status": status,
            "from": from_date,
            "to": to_date,
            "limit": limit,
            "offset": offset,
        }
        params = {k: v for k, v in params.items() if v is not None}
        return self._client._get("/meetings", params=params)
    
    def get(self, meeting_id: str) -> Meeting:
        """Get a specific meeting by ID."""
        data = self._client._get(f"/meetings/{meeting_id}")
        return Meeting(**data)
    
    def create(
        self,
        title: str,
        participants: List[Dict[str, str]],
        scheduled_at: Optional[str] = None,
        duration_minutes: Optional[int] = None,
        settings: Optional[Dict[str, Any]] = None,
    ) -> Meeting:
        """Create a new meeting."""
        data = {
            "title": title,
            "participants": participants,
            "scheduled_at": scheduled_at,
            "duration_minutes": duration_minutes,
            "settings": settings,
        }
        data = {k: v for k, v in data.items() if v is not None}
        response = self._client._post("/meetings", data=data)
        return Meeting(**response)
    
    def update(self, meeting_id: str, **updates) -> Meeting:
        """Update a meeting."""
        response = self._client._put(f"/meetings/{meeting_id}", data=updates)
        return Meeting(**response)
    
    def delete(self, meeting_id: str) -> None:
        """Delete a meeting."""
        self._client._delete(f"/meetings/{meeting_id}")


class TranscriptionsClient:
    """Client for transcriptions API."""
    
    def __init__(self, client: Client):
        self._client = client
    
    def get(self, meeting_id: str) -> Transcription:
        """Get transcription for a meeting."""
        data = self._client._get(f"/meetings/{meeting_id}/transcription")
        return Transcription(**data)
    
    def search(
        self,
        query: str,
        org_id: Optional[str] = None,
        from_date: Optional[str] = None,
        to_date: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Search transcriptions."""
        data = {
            "query": query,
            "org_id": org_id,
            "from_date": from_date,
            "to_date": to_date,
        }
        data = {k: v for k, v in data.items() if v is not None}
        return self._client._post("/transcriptions/search", data=data)


class WorkflowsClient:
    """Client for workflows API."""
    
    def __init__(self, client: Client):
        self._client = client
    
    def list(self) -> List[Workflow]:
        """List all workflows."""
        data = self._client._get("/workflows")
        return [Workflow(**w) for w in data.get("workflows", [])]
    
    def create(
        self,
        name: str,
        trigger: Dict[str, Any],
        actions: List[Dict[str, Any]],
    ) -> Workflow:
        """Create a new workflow."""
        data = {
            "name": name,
            "trigger": trigger,
            "actions": actions,
        }
        response = self._client._post("/workflows", data=data)
        return Workflow(**response)
    
    def execute(self, workflow_id: str, data: Optional[Dict[str, Any]] = None) -> None:
        """Execute a workflow."""
        self._client._post(f"/workflows/{workflow_id}/execute", data=data)


class PredictionsClient:
    """Client for predictions API."""
    
    def __init__(self, client: Client):
        self._client = client
    
    def predict_outcome(
        self,
        meeting_type: str,
        scheduled_duration: int,
        participants: List[str],
        agenda_items: List[str],
    ) -> Dict[str, Any]:
        """Predict meeting outcome."""
        data = {
            "meeting_type": meeting_type,
            "scheduled_duration": scheduled_duration,
            "participants": participants,
            "agenda_items": agenda_items,
        }
        return self._client._post("/predict/outcome", data=data)
    
    def predict_engagement(
        self,
        meeting_id: str,
        participants: List[str],
    ) -> Dict[str, Any]:
        """Predict meeting engagement."""
        data = {
            "meeting_id": meeting_id,
            "participants": participants,
        }
        return self._client._post("/predict/engagement", data=data)


class TranslationsClient:
    """Client for translations API."""
    
    def __init__(self, client: Client):
        self._client = client
    
    def translate(
        self,
        text: str,
        target_language: str,
        source_language: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Translate text."""
        data = {
            "text": text,
            "target_language": target_language,
            "source_language": source_language,
        }
        return self._client._post("/translate", data=data)
    
    def detect_language(self, text: str) -> Dict[str, Any]:
        """Detect language of text."""
        return self._client._post("/detect-language", data={"text": text})
