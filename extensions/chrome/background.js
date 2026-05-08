// Ruflo Extension Background Script

const API_URL = 'https://api.ruflo.io';
let mediaRecorder = null;
let recordedChunks = [];

// Listen for messages from popup/content
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  switch (request.action) {
    case 'startRecording':
      startRecording(request.tabId);
      break;
    case 'stopRecording':
      stopRecording();
      break;
    case 'getRecordingStatus':
      sendResponse({ isRecording: !!mediaRecorder });
      break;
    case 'authenticate':
      authenticate(sendResponse);
      return true; // Async response
  }
});

async function startRecording(tabId) {
  try {
    // Capture tab audio
    const streamId = await chrome.tabCapture.getMediaStreamId({ 
      targetTabId: tabId 
    });
    
    const stream = await navigator.mediaDevices.getUserMedia({
      audio: {
        mandatory: {
          chromeMediaSource: 'tab',
          chromeMediaSourceId: streamId
        }
      }
    });

    mediaRecorder = new MediaRecorder(stream);
    recordedChunks = [];

    mediaRecorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        recordedChunks.push(event.data);
      }
    };

    mediaRecorder.onstop = async () => {
      const blob = new Blob(recordedChunks, { type: 'audio/webm' });
      await uploadRecording(blob);
      
      // Stop all tracks
      stream.getTracks().forEach(track => track.stop());
    };

    mediaRecorder.start(1000); // Collect 1-second chunks
    
    // Update icon
    chrome.action.setIcon({
      path: {
        '16': 'icons/recording16.png',
        '48': 'icons/recording48.png',
        '128': 'icons/recording128.png'
      }
    });

    // Notify popup
    chrome.runtime.sendMessage({ action: 'recordingStarted' });

  } catch (error) {
    console.error('Failed to start recording:', error);
    chrome.notifications.create({
      type: 'basic',
      iconUrl: 'icons/icon48.png',
      title: 'Ruflo',
      message: 'Failed to start recording. Please try again.'
    });
  }
}

function stopRecording() {
  if (mediaRecorder && mediaRecorder.state !== 'inactive') {
    mediaRecorder.stop();
    mediaRecorder = null;
    
    // Reset icon
    chrome.action.setIcon({
      path: {
        '16': 'icons/icon16.png',
        '48': 'icons/icon48.png',
        '128': 'icons/icon128.png'
      }
    });

    chrome.runtime.sendMessage({ action: 'recordingStopped' });
  }
}

async function uploadRecording(blob) {
  const formData = new FormData();
  formData.append('file', blob, 'meeting-recording.webm');
  formData.append('source', 'browser-extension');

  try {
    const token = await getAuthToken();
    
    const response = await fetch(`${API_URL}/api/audio/upload`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`
      },
      body: formData
    });

    if (response.ok) {
      chrome.notifications.create({
        type: 'basic',
        iconUrl: 'icons/icon48.png',
        title: 'Ruflo',
        message: 'Recording uploaded successfully! Transcription in progress.'
      });
    } else {
      throw new Error('Upload failed');
    }
  } catch (error) {
    console.error('Upload failed:', error);
    chrome.notifications.create({
      type: 'basic',
      iconUrl: 'icons/icon48.png',
      title: 'Ruflo',
      message: 'Failed to upload recording. Please try again.'
    });
  }
}

async function getAuthToken() {
  return new Promise((resolve) => {
    chrome.storage.local.get(['accessToken'], (result) => {
      resolve(result.accessToken);
    });
  });
}

function authenticate(sendResponse) {
  const clientId = 'YOUR_CLIENT_ID';
  const redirectUri = chrome.identity.getRedirectURL();
  const authUrl = `${API_URL}/api/auth/oauth/authorize?client_id=${clientId}&redirect_uri=${redirectUri}&response_type=token`;

  chrome.identity.launchWebAuthFlow({
    url: authUrl,
    interactive: true
  }, (redirectUrl) => {
    if (chrome.runtime.lastError || !redirectUrl) {
      sendResponse({ success: false, error: chrome.runtime.lastError });
      return;
    }

    // Extract token from redirect URL
    const url = new URL(redirectUrl);
    const token = url.hash.match(/access_token=([^&]+)/)?.[1];

    if (token) {
      chrome.storage.local.set({ accessToken: token }, () => {
        sendResponse({ success: true });
      });
    } else {
      sendResponse({ success: false, error: 'No token found' });
    }
  });
}
