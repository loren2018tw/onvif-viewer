import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  CameraInfo,
  DiscoveredCamera,
  ScanRange,
  FFmpegStatus,
  PreviewSession,
  StreamType,
} from "@/types/camera";

export const useCameraStore = defineStore("camera", () => {
  const cameras = ref<CameraInfo[]>([]);
  const discoveredCameras = ref<DiscoveredCamera[]>([]);
  const isDiscovering = ref(false);
  const isScanning = ref(false);
  const isLoadingStream = ref(false);
  const currentStreamUri = ref<string | null>(null);
  const previewUrl = ref<string | null>(null);
  const previewSessionId = ref<string | null>(null);
  const previewHasAudio = ref(false);
  const audioEnabled = ref(false);
  const isPreviewing = ref(false);
  const error = ref<string | null>(null);
  const ffmpegStatus = ref<FFmpegStatus | null>(null);
  let previewRequestId = 0;

  interface CachedPreview {
    streamUri: string;
    hasAudio: boolean;
    cachedAt: number;
  }

  const previewCache = new Map<string, CachedPreview>();
  const PREVIEW_CACHE_TTL_MS = 10 * 60 * 1000;

  function previewCacheKey(cameraId: string, streamType: StreamType) {
    return `${cameraId}:${streamType}`;
  }

  function getCachedPreview(key: string): CachedPreview | null {
    const cached = previewCache.get(key);
    if (!cached) {
      return null;
    }
    if (Date.now() - cached.cachedAt > PREVIEW_CACHE_TTL_MS) {
      previewCache.delete(key);
      return null;
    }
    return cached;
  }

  function invalidateCameraPreviewCache(cameraId: string) {
    const prefix = `${cameraId}:`;
    for (const key of [...previewCache.keys()]) {
      if (key.startsWith(prefix)) {
        previewCache.delete(key);
      }
    }
  }

  function clearPreviewState() {
    currentStreamUri.value = null;
    previewUrl.value = null;
    previewSessionId.value = null;
    previewHasAudio.value = false;
    audioEnabled.value = false;
    isPreviewing.value = false;
    isLoadingStream.value = false;
  }

  async function loadCameras() {
    try {
      error.value = null;
      cameras.value = await invoke<CameraInfo[]>("list_cameras");
    } catch (e) {
      error.value = String(e);
    }
  }

  async function discoverCameras(durationSecs?: number) {
    try {
      isDiscovering.value = true;
      error.value = null;
      discoveredCameras.value = await invoke<DiscoveredCamera[]>(
        "discover_onvif_cameras",
        {
          durationSecs: durationSecs ?? 5,
        },
      );
    } catch (e) {
      error.value = String(e);
    } finally {
      isDiscovering.value = false;
    }
  }

  async function scanRange(range: ScanRange) {
    try {
      isScanning.value = true;
      error.value = null;
      discoveredCameras.value = await invoke<DiscoveredCamera[]>(
        "scan_onvif_range",
        { range },
      );
    } catch (e) {
      error.value = String(e);
    } finally {
      isScanning.value = false;
    }
  }

  async function saveCamera(
    name: string,
    address: string,
    onvifPort: number,
    username: string,
    password: string,
    streamUri?: string,
    manufacturer?: string,
    model?: string,
  ) {
    try {
      error.value = null;
      cameras.value = await invoke<CameraInfo[]>("save_camera", {
        name,
        address,
        onvifPort,
        username,
        password,
        streamUri: streamUri ?? null,
        manufacturer: manufacturer ?? null,
        model: model ?? null,
      });
    } catch (e) {
      error.value = String(e);
    }
  }

  async function deleteCamera(id: string) {
    try {
      error.value = null;
      cameras.value = await invoke<CameraInfo[]>("delete_camera", { id });
      invalidateCameraPreviewCache(id);
      if (currentStreamUri.value) {
        currentStreamUri.value = null;
      }
    } catch (e) {
      error.value = String(e);
    }
  }

  async function getStreamUri(
    address: string,
    port: number,
    username: string,
    password: string,
    streamType: StreamType = "sub",
  ) {
    try {
      isLoadingStream.value = true;
      error.value = null;
      currentStreamUri.value = await invoke<string>("get_stream_uri", {
        address,
        port,
        username,
        password,
        streamType,
      });
    } catch (e) {
      error.value = String(e);
      currentStreamUri.value = null;
    } finally {
      isLoadingStream.value = false;
    }
  }

  async function updateCamera(
    id: string,
    name: string,
    address: string,
    onvifPort: number,
    username: string,
    password: string,
  ) {
    try {
      error.value = null;
      cameras.value = await invoke<CameraInfo[]>("update_camera_info", {
        camera: {
          id,
          name,
          address,
          onvif_port: onvifPort,
          username,
          password,
          stream_uri: null,
          manufacturer: null,
          model: null,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      });
      invalidateCameraPreviewCache(id);
    } catch (e) {
      error.value = String(e);
    }
  }

  async function testConnection(
    address: string,
    port: number,
    username: string,
    password: string,
  ) {
    await invoke<void>("test_camera_connection", {
      address,
      port,
      username,
      password,
    });
  }

  async function diagnoseCamera(
    address: string,
    port: number,
    username: string,
    password: string,
  ) {
    return await invoke<string>("diagnose_camera", {
      address,
      port,
      username,
      password,
    });
  }

  function clearStream() {
    clearPreviewState();
  }

  async function startPreview(
    address: string,
    port: number,
    username: string,
    password: string,
    streamType: StreamType = "sub",
    cameraId?: string,
  ) {
    const requestId = ++previewRequestId;
    const cacheKey = cameraId ? previewCacheKey(cameraId, streamType) : null;
    const cached = cacheKey ? getCachedPreview(cacheKey) : null;

    currentStreamUri.value = null;
    previewUrl.value = null;
    previewSessionId.value = null;
    previewHasAudio.value = false;
    audioEnabled.value = false;
    isPreviewing.value = false;
    isLoadingStream.value = true;

    const invokeStart = (useCache: boolean) =>
      invoke<PreviewSession>("start_preview", {
        address,
        port,
        username,
        password,
        streamType,
        streamUri: useCache ? (cached?.streamUri ?? null) : null,
        hasAudio: useCache ? (cached?.hasAudio ?? null) : null,
        cameraId: cameraId ?? null,
      });

    try {
      error.value = null;
      let preview: PreviewSession;
      try {
        preview = await invokeStart(true);
      } catch (firstError) {
        if (requestId !== previewRequestId) {
          return;
        }
        // A cached URI can become stale (camera rebooted / profile changed).
        // Drop it and resolve a fresh URI once before surfacing the error.
        if (!cached || !cacheKey) {
          throw firstError;
        }
        previewCache.delete(cacheKey);
        preview = await invokeStart(false);
      }
      if (requestId !== previewRequestId) {
        return;
      }

      previewUrl.value = preview.preview_url;
      previewSessionId.value = preview.session_id;
      currentStreamUri.value = preview.stream_uri;
      previewHasAudio.value = preview.has_audio;

      if (cacheKey) {
        previewCache.set(cacheKey, {
          streamUri: preview.stream_uri,
          hasAudio: preview.has_audio,
          cachedAt: Date.now(),
        });
      }
    } catch (e) {
      if (requestId !== previewRequestId) {
        return;
      }
      error.value = String(e);
      isLoadingStream.value = false;
      clearPreviewState();
    }
  }

  async function stopPreview() {
    previewRequestId += 1;
    clearPreviewState();
    try {
      await invoke("stop_preview");
    } catch (e) {
      error.value = String(e);
    }
  }

  function markPreviewReady(sessionId: string) {
    if (sessionId !== previewSessionId.value) {
      return;
    }
    isLoadingStream.value = false;
    isPreviewing.value = true;
  }

  function markPreviewFailed(sessionId?: string, message?: string) {
    if (sessionId && sessionId !== previewSessionId.value) {
      return;
    }
    if (message) {
      error.value = message;
    }
    clearPreviewState();
  }

  function setAudioEnabled(enabled: boolean) {
    audioEnabled.value = previewHasAudio.value ? enabled : false;
  }

  function clearError() {
    error.value = null;
  }

  async function checkFFmpeg() {
    try {
      ffmpegStatus.value = await invoke<FFmpegStatus>("check_ffmpeg");
    } catch (e) {
      error.value = String(e);
    }
  }

  return {
    cameras,
    discoveredCameras,
    isDiscovering,
    isScanning,
    isLoadingStream,
    currentStreamUri,
    previewUrl,
    previewSessionId,
    previewHasAudio,
    audioEnabled,
    isPreviewing,
    error,
    ffmpegStatus,
    loadCameras,
    discoverCameras,
    scanRange,
    saveCamera,
    deleteCamera,
    updateCamera,
    getStreamUri,
    testConnection,
    diagnoseCamera,
    startPreview,
    stopPreview,
    markPreviewReady,
    markPreviewFailed,
    setAudioEnabled,
    clearStream,
    clearError,
    checkFFmpeg,
  };
});
