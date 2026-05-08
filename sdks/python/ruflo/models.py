"""Data models for Ruflo SDK."""

from dataclasses import dataclass
from typing import Optional, List, Dict, Any
from datetime import datetime


@dataclass
class Participant:
    """Meeting participant."""
    id: str
    name: str
    email: str
    role: Optional[str] = None
    joined_at: Optional[str] = None
    left_at: Optional[str] = None


@dataclass
class Meeting:
    """Meeting object."""
    id: str
    title: str
    status: str
    participants: List[Dict[str, Any]]
    started_at: Optional[str] = None
    ended_at: Optional[str] = None
    duration_seconds: Optional[int] = None
    recording_url: Optional[str] = None
    transcription_status: Optional[str] = None


@dataclass
class TranscriptionSegment:
    """Transcription segment."""
    id: str
    speaker: str
    text: str
    start_time: float
    end_time: float
    confidence: float


@dataclass
class Transcription:
    """Meeting transcription."""
    meeting_id: str
    language: str
    segments: List[TranscriptionSegment]
    full_text: str


@dataclass
class ActionItem:
    """Action item extracted from meeting."""
    id: str
    text: str
    priority: str
    completed: bool
    assignee: Optional[str] = None
    due_date: Optional[str] = None


@dataclass
class MeetingAnalysis:
    """Meeting analysis results."""
    meeting_id: str
    sentiment: Dict[str, Any]
    action_items: List[ActionItem]
    decisions: List[Dict[str, Any]]
    topics: List[Dict[str, Any]]
    speakers: List[Dict[str, Any]]


@dataclass
class Workflow:
    """Automation workflow."""
    id: str
    name: str
    enabled: bool
    trigger: Dict[str, Any]
    actions: List[Dict[str, Any]]


@dataclass
class TranslationResult:
    """Translation result."""
    translated_text: str
    source_language: str
    target_language: str
    confidence: float


@dataclass
class PredictionResult:
    """Prediction result."""
    success_probability: float
    confidence: float
    factors: List[Dict[str, Any]]
    recommendations: List[str]
