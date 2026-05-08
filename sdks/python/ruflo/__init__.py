"""
Ruflo Python SDK - Official Python SDK for Ruflo AI Meeting Notes Platform

Usage:
    import ruflo
    
    client = ruflo.Client(api_key="your-api-key")
    
    # List meetings
    meetings = client.meetings.list()
    
    # Get transcription
    transcription = client.transcriptions.get(meeting_id="mtg_123")
    
    # Create workflow
    workflow = client.workflows.create(
        name="Post-meeting Summary",
        trigger={"type": "meeting.ended"},
        actions=[{"type": "slack.send_message", "config": {"channel": "#meetings"}}]
    )
"""

from .client import Client
from .exceptions import RufloError, RufloAPIError, RufloAuthError
from .models import (
    Meeting,
    Participant,
    Transcription,
    TranscriptionSegment,
    ActionItem,
    MeetingAnalysis,
    Workflow,
)

__version__ = "1.0.0"
__all__ = [
    "Client",
    "RufloError",
    "RufloAPIError",
    "RufloAuthError",
    "Meeting",
    "Participant",
    "Transcription",
    "TranscriptionSegment",
    "ActionItem",
    "MeetingAnalysis",
    "Workflow",
]
