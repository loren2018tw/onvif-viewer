import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  CameraInfo,
  DiscoveredCamera,
  ScanRange,
  FFmpegStatus,
} from "@/types/camera";

export const useCameraStore = defineStore("camera", () => {
  const cameras = ref<CameraInfo[]>([]);
  const discoveredCameras = ref<DiscoveredCamera[]>([]);
  const isDiscovering = ref(false);
  const isScanning = ref(false);
  const isLoadingStream = ref(false);
  const currentStreamUri = ref<string | null>(null);
  const previewUrl = ref<string | null>(null);
  const isPreviewing = ref(false);
  const error = ref<string | null>(null);
  const ffmpegStatus = ref<FFmpegStatus | null>(null);

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
  ) {
    try {
      isLoadingStream.value = true;
      error.value = null;
      currentStreamUri.value = await invoke<string>("get_stream_uri", {
        address,
        port,
        username,
        password,
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
    currentStreamUri.value = null;
    previewUrl.value = null;
    isPreviewing.value = false;
  }

  async function startPreview(
    address: string,
    port: number,
    username: string,
    password: string,
  ) {
    try {
      isLoadingStream.value = true;
      error.value = null;
      const url = await invoke<string>("start_preview", {
        address,
        port,
        username,
        password,
      });
      previewUrl.value = url;
      isPreviewing.value = true;
    } catch (e) {
      error.value = String(e);
      previewUrl.value = null;
      isPreviewing.value = false;
    } finally {
      isLoadingStream.value = false;
    }
  }

  async function stopPreview() {
    try {
      await invoke("stop_preview");
    } catch (e) {
      error.value = String(e);
    } finally {
      previewUrl.value = null;
      isPreviewing.value = false;
    }
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
    clearStream,
    clearError,
    checkFFmpeg,
  };
});
