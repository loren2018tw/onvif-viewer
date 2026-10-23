<template>
  <v-container fluid class="home-container">
    <v-row class="home-row">
      <!-- Left panel: Camera list & discovery -->
      <v-col cols="12" lg="4" class="left-panel">
        <v-card class="left-panel-card glass-panel">
          <div class="cx-panel-strip">
            <span class="cx-hud-label">NODE REGISTRY</span>
            <span class="cx-strip-meta">ONVIF / DISCOVERY</span>
          </div>
          <div class="left-panel-header d-flex align-center pr-2">
            <v-tabs v-model="leftTab" density="compact" class="flex-grow-1">
              <v-tab value="saved">已儲存攝影機</v-tab>
              <v-tab value="search">搜尋攝影機</v-tab>
            </v-tabs>
            <v-btn
              icon
              size="small"
              aria-label="新增攝影機"
              title="新增攝影機"
              @click="openAddDialog()"
            >
              <v-icon>mdi-plus</v-icon>
            </v-btn>
          </div>

          <!-- Saved cameras -->
          <div v-show="leftTab === 'saved'" class="tab-page">
            <div class="px-4 pt-3 pb-2 d-flex align-center">
              <v-text-field
                v-model="savedCameraFilter"
                label="快速篩選"
                placeholder="輸入名稱、位址或連接埠"
                density="compact"
                hide-details
                clearable
                prepend-inner-icon="mdi-filter-variant"
              />
              <span class="cx-count ml-3">
                {{ savedCountLabel }}
              </span>
            </div>
            <div class="list-scroll">
              <v-list v-if="filteredSavedCameras.length > 0" density="compact">
                <v-list-item
                  v-for="cam in filteredSavedCameras"
                  :key="cam.id"
                  :active="selectedCamera?.id === cam.id"
                  @click="selectCamera(cam)"
                >
                  <template #prepend>
                    <v-icon>mdi-camera</v-icon>
                  </template>
                  <v-list-item-title>{{ cam.name }}</v-list-item-title>
                  <v-list-item-subtitle
                    >{{ cam.address }}:{{
                      cam.onvif_port
                    }}</v-list-item-subtitle
                  >
                  <template #append>
                    <v-btn
                      icon
                      size="x-small"
                      color="primary"
                      class="mr-1"
                      @click.stop="openEditDialog(cam)"
                    >
                      <v-icon>mdi-pencil</v-icon>
                    </v-btn>
                    <v-btn
                      icon
                      size="x-small"
                      color="error"
                      @click.stop="handleDelete(cam.id)"
                    >
                      <v-icon>mdi-delete</v-icon>
                    </v-btn>
                  </template>
                </v-list-item>
              </v-list>
              <v-card-text v-else class="text-center text-medium-emphasis">
                {{
                  store.cameras.length === 0
                    ? "尚無儲存的攝影機"
                    : "找不到符合篩選條件的攝影機"
                }}
              </v-card-text>
            </div>
          </div>

          <!-- Discovered cameras -->
          <div v-show="leftTab === 'search'" class="tab-page">
            <div class="px-4 pt-3 pb-2 flex-shrink-0">
              <v-btn-toggle
                v-model="searchTab"
                mandatory
                density="compact"
                divided
                variant="outlined"
                class="w-100 mb-3"
              >
                <v-btn value="auto" class="flex-grow-1">自動搜尋</v-btn>
                <v-btn value="manual" class="flex-grow-1">手動掃描</v-btn>
              </v-btn-toggle>

              <div v-if="searchTab === 'auto'">
                <v-btn
                  block
                  color="primary"
                  :loading="store.isDiscovering"
                  @click="store.discoverCameras(5)"
                >
                  <v-icon class="mr-1">mdi-radar</v-icon>
                  搜尋本地攝影機
                </v-btn>
              </div>

              <div v-else>
                <v-text-field
                  v-model="scanStartIp"
                  label="起始 IP"
                  placeholder="192.168.1.1"
                  density="compact"
                  class="mb-2"
                />
                <v-text-field
                  v-model="scanEndIp"
                  label="結束 IP"
                  placeholder="192.168.1.254"
                  density="compact"
                  class="mb-2"
                />
                <v-text-field
                  v-model.number="scanPort"
                  label="ONVIF 連接埠"
                  type="number"
                  density="compact"
                  class="mb-2"
                />
                <v-btn
                  block
                  color="primary"
                  :loading="store.isScanning"
                  :disabled="!scanStartIp || !scanEndIp"
                  @click="handleScanRange"
                >
                  <v-icon class="mr-1">mdi-ip-network</v-icon>
                  掃描範圍
                </v-btn>
              </div>
            </div>

            <div class="px-4 pb-2 d-flex align-center flex-shrink-0">
              <span class="cx-result-label">
                搜尋結果
                <b>{{ store.discoveredCameras.length }}</b>
              </span>
              <v-spacer />
              <span
                v-if="hiddenSavedCount > 0"
                class="text-caption text-medium-emphasis mr-1"
              >
                已隱藏 {{ hiddenSavedCount }} 台
              </span>
              <v-checkbox
                v-model="hideSavedCameras"
                label="隱藏已儲存"
                density="compact"
                hide-details
                class="flex-grow-0"
              />
            </div>

            <div class="list-scroll">
              <v-list
                v-if="visibleDiscoveredCameras.length > 0"
                density="compact"
              >
                <v-list-item
                  v-for="(cam, idx) in visibleDiscoveredCameras"
                  :key="idx"
                  @click="openAddDialog(cam)"
                >
                  <template #prepend>
                    <v-icon color="success">mdi-camera</v-icon>
                  </template>
                  <v-list-item-title>{{
                    cam.name || cam.address
                  }}</v-list-item-title>
                  <v-list-item-subtitle
                    >{{ cam.address }}:{{ cam.port }}</v-list-item-subtitle
                  >
                  <template #append>
                    <v-btn
                      icon
                      size="small"
                      color="primary"
                      @click.stop="openAddDialog(cam)"
                    >
                      <v-icon>mdi-plus</v-icon>
                    </v-btn>
                  </template>
                </v-list-item>
              </v-list>
              <v-card-text v-else class="text-center text-medium-emphasis">
                {{
                  store.discoveredCameras.length === 0
                    ? "尚無搜尋結果"
                    : "搜尋結果皆已儲存"
                }}
              </v-card-text>
            </div>
          </div>
        </v-card>
      </v-col>

      <!-- Right panel: Preview -->
      <v-col cols="12" lg="8" class="right-panel">
        <v-card class="preview-card glass-panel">
          <v-card-title
            class="d-flex align-center flex-wrap preview-header"
            style="row-gap: 8px; column-gap: 8px"
          >
            <div class="preview-heading">
              <span class="preview-heading-icon">
                <v-icon>mdi-monitor</v-icon>
              </span>
              <span class="preview-heading-text">
                <span class="cx-hud-label">LIVE FEED</span>
                <span class="preview-heading-title">即時預覽</span>
              </span>
              <span v-if="store.isPreviewing" class="cx-live-badge">
                <span class="cx-live-dot"></span>
                LIVE
              </span>
              <span
                v-else-if="store.isLoadingStream"
                class="cx-live-badge is-pending"
              >
                <span class="cx-live-dot"></span>
                LINKING
              </span>
            </div>
            <div class="preview-toolbar">
              <v-select
                v-model="selectedStreamType"
                :items="streamTypeOptions"
                item-title="title"
                item-value="value"
                label="碼流"
                density="compact"
                hide-details
                variant="outlined"
                class="stream-select"
              />
              <v-checkbox
                v-model="autoPreview"
                label="自動預覽"
                density="compact"
                hide-details
                class="flex-grow-0"
              />
              <v-switch
                v-if="store.previewHasAudio"
                v-model="audioPlaybackEnabled"
                label="播放聲音"
                color="primary"
                density="compact"
                hide-details
                inset
                class="flex-grow-0"
              />
              <v-chip
                v-if="selectedCamera"
                color="primary"
                size="small"
                class="cx-camera-chip"
              >
                {{ selectedCamera.name }}
              </v-chip>
              <v-btn
                v-if="selectedCamera"
                color="primary"
                size="small"
                :loading="store.isLoadingStream"
                @click="handlePreview"
              >
                <v-icon class="mr-1">mdi-play</v-icon>
                預覽
              </v-btn>
              <v-btn
                v-if="store.isPreviewing || store.isLoadingStream"
                color="error"
                size="small"
                @click="handleStopPreview"
              >
                <v-icon class="mr-1">mdi-stop</v-icon>
                停止
              </v-btn>
            </div>
          </v-card-title>
          <v-card-text class="preview-body d-flex justify-center align-center">
            <div
              v-if="store.isLoadingStream && !store.previewUrl"
              class="cx-boot-frame"
            >
              <img
                v-if="previewPoster"
                :src="previewPoster"
                alt=""
                class="cx-boot-poster"
              />
              <div class="text-center cx-boot cx-boot-overlay">
                <div class="cx-boot-ring">
                  <v-progress-circular indeterminate color="primary" size="64" />
                </div>
                <div class="cx-boot-title">ESTABLISHING UPLINK</div>
                <div class="mt-2 text-medium-emphasis">
                  {{
                    previewPoster
                      ? "顯示上次畫面，正在重新連線..."
                      : "正在連線攝影機..."
                  }}
                </div>
              </div>
            </div>
            <div
              v-else-if="store.previewUrl"
              class="w-100 h-100 preview-content"
            >
              <div
                class="d-flex justify-center align-center preview-frame preview-stage cx-preview-frame"
              >
                <span class="cx-corner cx-tl" aria-hidden="true"></span>
                <span class="cx-corner cx-tr" aria-hidden="true"></span>
                <span class="cx-corner cx-bl" aria-hidden="true"></span>
                <span class="cx-corner cx-br" aria-hidden="true"></span>
                <span class="cx-preview-grid" aria-hidden="true"></span>
                <img
                  v-if="previewPoster && !isPreviewVisible"
                  :src="previewPoster"
                  alt=""
                  class="preview-poster"
                />
                <video
                  ref="previewVideo"
                  autoplay
                  playsinline
                  controls
                  crossorigin="anonymous"
                  class="preview-video"
                  :class="{ 'preview-video-hidden': !isPreviewVisible }"
                  @canplay="onPreviewReady"
                  @loadeddata="onPreviewReady"
                  @error="onPreviewPlaybackError"
                />
                <span class="cx-preview-scan" aria-hidden="true"></span>
                <div
                  v-if="store.isLoadingStream"
                  class="preview-overlay text-center cx-loading-overlay"
                >
                  <v-progress-circular
                    indeterminate
                    color="primary"
                    size="56"
                  />
                  <div class="cx-loading-title">ESTABLISHING UPLINK</div>
                  <div class="mt-1 text-medium-emphasis">
                    {{
                      previewPoster
                        ? "顯示上次畫面，正在重新連線..."
                        : "正在連線攝影機..."
                    }}
                  </div>
                </div>
              </div>
              <div v-if="store.currentStreamUri" class="mt-3">
                <v-alert
                  type="info"
                  variant="tonal"
                  density="compact"
                  class="cx-stream-readout"
                >
                  <div class="d-flex align-center">
                    <span class="cx-stream-tag">URI</span>
                    <code
                      class="text-caption flex-grow-1 cx-stream-uri"
                      style="white-space: nowrap; overflow-x: auto"
                      >{{ store.currentStreamUri }}</code
                    >
                    <v-btn
                      icon
                      size="x-small"
                      class="ml-2"
                      @click="copyStreamUri"
                    >
                      <v-icon>mdi-content-copy</v-icon>
                    </v-btn>
                  </div>
                </v-alert>
              </div>
            </div>
            <div
              v-else-if="selectedCamera && previewPoster"
              class="w-100 h-100 preview-content"
            >
              <div
                class="d-flex justify-center align-center preview-frame preview-stage cx-preview-frame"
              >
                <span class="cx-corner cx-tl" aria-hidden="true"></span>
                <span class="cx-corner cx-tr" aria-hidden="true"></span>
                <span class="cx-corner cx-bl" aria-hidden="true"></span>
                <span class="cx-corner cx-br" aria-hidden="true"></span>
                <span class="cx-preview-grid" aria-hidden="true"></span>
                <img
                  :src="previewPoster"
                  alt=""
                  class="preview-poster preview-poster-static"
                />
                <div class="cx-static-overlay">
                  <div class="cx-static-badge">LAST FRAME</div>
                  <div class="cx-static-hint">
                    上次預覽畫面 · 點選「預覽」開始直播
                  </div>
                </div>
              </div>
            </div>
            <div v-else class="text-center text-medium-emphasis cx-empty-state">
              <div class="cx-empty-ring">
                <v-icon size="52">mdi-camera-off</v-icon>
              </div>
              <div class="cx-empty-title">NO SIGNAL</div>
              <div class="cx-empty-hint">請選擇一個攝影機並點選「預覽」</div>
              <div class="cx-empty-meta">STANDBY · AWAITING NODE</div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Error snackbar -->
    <v-snackbar v-model="showError" color="error" timeout="10000">
      {{ store.error }}
      <template #actions>
        <v-btn
          variant="text"
          @click="
            store.clearError();
            showError = false;
          "
          >關閉</v-btn
        >
      </template>
    </v-snackbar>

    <!-- Copy success snackbar -->
    <v-snackbar v-model="showCopySuccess" color="success" timeout="2000">
      已複製到剪貼簿
    </v-snackbar>

    <!-- Test connection result snackbar -->
    <v-snackbar
      v-model="showTestResult"
      :color="testResultColor"
      timeout="4000"
    >
      {{ testResultMessage }}
      <template #actions>
        <v-btn variant="text" @click="showTestResult = false">關閉</v-btn>
      </template>
    </v-snackbar>

    <!-- FFmpeg not installed dialog -->
    <v-dialog v-model="ffmpegDialogOpen" max-width="500">
      <v-card class="glass-panel cx-dialog">
        <v-card-title class="d-flex align-center">
          <v-icon class="mr-2" color="warning">mdi-alert-circle</v-icon>
          未偵測到 FFmpeg
        </v-card-title>
        <v-card-text>
          <div class="mb-4">部分功能需要 FFmpeg。請使用以下指令安裝：</div>
          <v-alert
            v-if="store.ffmpegStatus"
            type="info"
            variant="tonal"
            class="mb-3"
          >
            <code style="word-break: break-all">{{
              store.ffmpegStatus.install_command
            }}</code>
          </v-alert>
          <div class="text-caption text-medium-emphasis">
            安裝後請重新啟動應用程式。
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="ffmpegDialogOpen = false" variant="tonal">
            稍後提醒
          </v-btn>
          <v-btn @click="ffmpegDialogOpen = false" color="primary">
            了解
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Diagnose result dialog -->
    <v-dialog v-model="diagnoseDialog" max-width="700">
      <v-card class="glass-panel cx-dialog">
        <v-card-title>
          <v-icon class="mr-2">mdi-stethoscope</v-icon>
          ONVIF 診斷報告
        </v-card-title>
        <v-card-text>
          <v-sheet
            rounded
            class="text-body-2 pa-3"
            style="
              white-space: pre-wrap;
              word-break: break-all;
              font-family: monospace;
            "
            color="surface-variant"
          >
            <pre class="ma-0">{{ diagnoseReport }}</pre>
          </v-sheet>
        </v-card-text>
        <v-card-actions>
          <v-btn variant="tonal" @click="copyDiagnoseReport">
            <v-icon class="mr-1">mdi-content-copy</v-icon>
            複製
          </v-btn>
          <v-spacer />
          <v-btn @click="diagnoseDialog = false">關閉</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Add/Edit camera dialog -->
    <v-dialog v-model="addDialog" max-width="500">
      <v-card class="glass-panel cx-dialog">
        <v-card-title>
          <v-icon class="mr-2">{{
            editingCameraId ? "mdi-pencil" : "mdi-camera-plus"
          }}</v-icon>
          {{ editingCameraId ? "編輯攝影機" : "新增攝影機" }}
        </v-card-title>
        <v-card-text>
          <v-text-field
            v-model="formData.name"
            label="名稱"
            density="compact"
            class="mb-2"
          />
          <v-text-field
            v-model="formData.address"
            label="IP 位址"
            density="compact"
            class="mb-2"
          />
          <v-text-field
            v-model.number="formData.port"
            label="ONVIF 連接埠"
            type="number"
            density="compact"
            class="mb-2"
          />
          <v-text-field
            v-model="formData.username"
            label="使用者名稱"
            density="compact"
            class="mb-2"
          />
          <v-text-field
            v-model="formData.password"
            label="密碼"
            :type="showPassword ? 'text' : 'password'"
            density="compact"
            class="mb-2"
            :append-inner-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'"
            @click:append-inner="showPassword = !showPassword"
          />
        </v-card-text>
        <v-card-actions>
          <v-btn
            color="info"
            variant="tonal"
            :loading="isTesting"
            :disabled="!formData.address || !formData.username"
            @click="handleTestConnection"
          >
            <v-icon class="mr-1">mdi-connection</v-icon>
            測試連線
          </v-btn>
          <v-btn
            color="warning"
            variant="tonal"
            :loading="isDiagnosing"
            :disabled="!formData.address || !formData.username"
            @click="handleDiagnose"
          >
            <v-icon class="mr-1">mdi-stethoscope</v-icon>
            診斷
          </v-btn>
          <v-spacer />
          <v-btn @click="addDialog = false">取消</v-btn>
          <v-btn
            color="primary"
            :disabled="!formData.name || !formData.address"
            @click="handleSaveCamera"
          >
            儲存
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script setup lang="ts">
import Hls from "hls.js";
import { invoke } from "@tauri-apps/api/core";
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import { useCameraStore } from "@/stores/camera";
import type { CameraInfo, DiscoveredCamera, StreamType } from "@/types/camera";

const store = useCameraStore();

const searchTab = ref("auto");
const leftTab = ref("saved");
const scanStartIp = ref("192.168.1.1");
const scanEndIp = ref("192.168.1.254");
const scanPort = ref(80);
// Vuetify 的 clearable 會把 modelValue 設成 null；篩選字串的契約是「永遠是 string」
const savedCameraFilterText = ref("");
const savedCameraFilter = computed({
  get: (): string => savedCameraFilterText.value,
  set: (value: string | null) => {
    savedCameraFilterText.value = value ?? "";
  },
});
const selectedCamera = ref<CameraInfo | null>(null);
const selectedStreamType = ref<StreamType>("sub");
const autoPreview = ref(false);
const addDialog = ref(false);
const showError = ref(false);
const showCopySuccess = ref(false);

const showPassword = ref(false);
const isTesting = ref(false);
const showTestResult = ref(false);
const testResultColor = ref("success");
const testResultMessage = ref("");
const lastCredentials = ref({ username: "admin", password: "" });
const editingCameraId = ref<string | null>(null);
const isDiagnosing = ref(false);
const diagnoseDialog = ref(false);
const diagnoseReport = ref("");

const formData = ref({
  name: "",
  address: "",
  port: 80,
  username: "admin",
  password: "",
});

const ffmpegDialogOpen = ref(false);
const previewVideo = ref<HTMLVideoElement | null>(null);
const previewPoster = ref<string | null>(null);
const posterCache = new Map<string, string>();
const isPreviewVisible = ref(false);
const streamTypeOptions: { title: string; value: StreamType }[] = [
  { title: "副碼流", value: "sub" },
  { title: "主碼流", value: "main" },
];
const audioPlaybackEnabled = computed({
  get: () => store.audioEnabled,
  set: (value: boolean) => {
    store.setAudioEnabled(value);
    applyAudioPreference();
  },
});
const filteredSavedCameras = computed(() => {
  const keyword = savedCameraFilter.value.trim().toLowerCase();
  if (!keyword) {
    return store.cameras;
  }

  return store.cameras.filter((cam) => {
    const searchableText = `${cam.name} ${cam.address} ${cam.onvif_port}`.toLowerCase();
    return searchableText.includes(keyword);
  });
});

const savedCountLabel = computed(() => {
  const total = store.cameras.length;
  const visible = filteredSavedCameras.value.length;
  return visible === total ? `共 ${total} 台` : `${visible} / ${total} 台`;
});

const hideSavedCameras = ref(false);

// 與後端 storage.rs 的去重鍵一致：address + port
function isSavedCamera(cam: DiscoveredCamera) {
  return store.cameras.some(
    (saved) => saved.address === cam.address && saved.onvif_port === cam.port,
  );
}

const visibleDiscoveredCameras = computed(() =>
  hideSavedCameras.value
    ? store.discoveredCameras.filter((cam) => !isSavedCamera(cam))
    : store.discoveredCameras,
);

const hiddenSavedCount = computed(() =>
  hideSavedCameras.value
    ? store.discoveredCameras.length - visibleDiscoveredCameras.value.length
    : 0,
);

let hls: Hls | null = null;
let attachedSessionId: string | null = null;
let playbackTimeoutId: ReturnType<typeof setTimeout> | null = null;
const PLAYBACK_START_TIMEOUT_MS = 20000;

function clearPlaybackTimeout() {
  if (playbackTimeoutId !== null) {
    clearTimeout(playbackTimeoutId);
    playbackTimeoutId = null;
  }
}

function armPlaybackTimeout(sessionId: string) {
  clearPlaybackTimeout();
  playbackTimeoutId = setTimeout(() => {
    playbackTimeoutId = null;
    if (
      attachedSessionId === sessionId &&
      store.previewSessionId === sessionId &&
      !store.isPreviewing
    ) {
      failPreviewPlayback(
        sessionId,
        "預覽播放逾時，影片解碼器可能未正確載入。若持續發生，請檢查系統的 H.264 解碼支援。",
      );
    }
  }, PLAYBACK_START_TIMEOUT_MS);
}

function canUseHlsJs() {
  if (!Hls.isSupported() || typeof MediaSource === "undefined") {
    return false;
  }

  try {
    return MediaSource.isTypeSupported('video/mp4; codecs="avc1.42E01E"');
  } catch {
    return false;
  }
}

function playbackUnsupportedMessage() {
  if (navigator.userAgent.includes("Linux")) {
    return "無法播放 H.264 串流：WebView 缺少影音解碼外掛。請安裝 gstreamer1.0-libav、gstreamer1.0-plugins-bad、gstreamer1.0-plugins-ugly 後重新啟動應用程式。";
  }
  return "目前的系統不支援 H.264 串流播放（缺少 Media Source 解碼支援）。";
}

function failPreviewPlayback(sessionId: string | undefined, message: string) {
  if (sessionId && store.previewSessionId !== sessionId) {
    return;
  }

  destroyPreviewPlayer();
  store.markPreviewFailed(sessionId, message);
  if (sessionId) {
    store.stopPreview();
  }
}

async function loadCachedSnapshot(cameraId: string, force = false) {
  if (!force && posterCache.has(cameraId)) {
    if (selectedCamera.value?.id === cameraId) {
      previewPoster.value = posterCache.get(cameraId) ?? null;
    }
    return;
  }

  try {
    const dataUrl = await invoke<string | null>("get_camera_snapshot", {
      cameraId,
    });
    if (dataUrl) {
      posterCache.set(cameraId, dataUrl);
    } else {
      posterCache.delete(cameraId);
    }
    if (selectedCamera.value?.id === cameraId) {
      previewPoster.value = dataUrl;
    }
  } catch {
    // 讀取快取失敗時保留目前畫面即可，不影響預覽流程。
  }
}

function showCachedFrame(cameraId: string) {
  previewPoster.value = posterCache.get(cameraId) ?? null;
  void loadCachedSnapshot(cameraId, true);
}

watch(
  () => store.error,
  (val) => {
    if (val) showError.value = true;
  },
);

watch(selectedStreamType, () => {
  // 若正在預覽（含連線載入中）或開啟了自動預覽，切換碼流時自動重啟
  const isActive = store.isPreviewing || store.isLoadingStream;
  if (selectedCamera.value && (isActive || autoPreview.value)) {
    handlePreview();
  }
});

onMounted(() => {
  store.loadCameras();
  store.checkFFmpeg().then(() => {
    if (store.ffmpegStatus && !store.ffmpegStatus.installed) {
      ffmpegDialogOpen.value = true;
    }
  });
});

onBeforeUnmount(() => {
  destroyPreviewPlayer();
  void store.stopPreview();
});

watch(
  () => [store.previewUrl, store.previewSessionId] as const,
  async ([previewUrl, sessionId]) => {
    destroyPreviewPlayer();

    if (!previewUrl || !sessionId) {
      return;
    }

    await nextTick();

    const video = previewVideo.value;
    if (!video) {
      failPreviewPlayback(sessionId, "無法建立預覽播放器，請重新嘗試。");
      return;
    }

    attachedSessionId = sessionId;
    applyAudioPreference();

    if (video.canPlayType("application/vnd.apple.mpegurl")) {
      video.src = previewUrl;
    } else if (canUseHlsJs()) {
      hls = new Hls({
        enableWorker: true,
        lowLatencyMode: true,
      });
      hls.on(Hls.Events.ERROR, (_event, data) => {
        if (!data.fatal) {
          return;
        }
        failPreviewPlayback(
          sessionId,
          `串流播放失敗（${data.type} / ${data.details}）`,
        );
      });
      hls.loadSource(previewUrl);
      hls.attachMedia(video);
    } else {
      // WebView 回報支援 MSE，但實際上無法解碼 H.264（例如 Linux 缺少
      // GStreamer 外掛）。hls.js 不會發出錯誤，只會停在載入中，因此這裡
      // 主動偵測並回報，避免畫面一直轉圈圈。
      failPreviewPlayback(sessionId, playbackUnsupportedMessage());
      return;
    }

    armPlaybackTimeout(sessionId);

    video.play().catch(() => {
      // Waiting for canplay or a user gesture is acceptable here.
    });
  },
);

function destroyPreviewPlayer() {
  clearPlaybackTimeout();
  isPreviewVisible.value = false;
  attachedSessionId = null;

  if (hls) {
    hls.destroy();
    hls = null;
  }

  const video = previewVideo.value;
  if (video) {
    video.pause();
    video.removeAttribute("src");
    video.load();
  }
}

function applyAudioPreference() {
  const video = previewVideo.value;
  if (!video) {
    return;
  }

  // 只負責設定靜音狀態，不自己呼叫 play()
  video.muted = !store.audioEnabled;
}

async function selectCamera(cam: CameraInfo) {
  const previousId = selectedCamera.value?.id;
  selectedCamera.value = cam;
  showCachedFrame(cam.id);
  if (autoPreview.value) {
    // 先停止舊的預覽再啟動新的，避免殘留狀態
    destroyPreviewPlayer();
    await store.stopPreview();
    if (previousId && previousId !== cam.id) {
      void loadCachedSnapshot(previousId, true);
    }
    await store.startPreview(
      cam.address,
      cam.onvif_port,
      cam.username,
      cam.password,
      selectedStreamType.value,
      cam.id,
    );
  } else {
    destroyPreviewPlayer();
    await store.stopPreview();
    if (previousId && previousId !== cam.id) {
      void loadCachedSnapshot(previousId, true);
    }
  }
}

async function handlePreview() {
  if (!selectedCamera.value) return;
  const cam = selectedCamera.value;
  showCachedFrame(cam.id);
  await store.startPreview(
    cam.address,
    cam.onvif_port,
    cam.username,
    cam.password,
    selectedStreamType.value,
    cam.id,
  );
}

async function handleStopPreview() {
  destroyPreviewPlayer();
  await store.stopPreview();
  if (selectedCamera.value) {
    await loadCachedSnapshot(selectedCamera.value.id, true);
  }
}

function onPreviewReady() {
  if (!store.previewSessionId || attachedSessionId !== store.previewSessionId) {
    return;
  }

  clearPlaybackTimeout();
  isPreviewVisible.value = true;
  store.markPreviewReady(store.previewSessionId);
  applyAudioPreference();

  // 確保對播放，不管是首次啟動還是切換碼流後都需要重新啟動
  const video = previewVideo.value;
  if (video && video.paused) {
    video.play().catch(() => {
      // 等待 canplay 或使用者手勢觸發是可接受的
    });
  }
}

function onPreviewPlaybackError() {
  failPreviewPlayback(
    store.previewSessionId ?? undefined,
    "影片播放失敗，請確認攝影機的 RTSP 串流是否正常。",
  );
}

function handleScanRange() {
  store.scanRange({
    start_ip: scanStartIp.value,
    end_ip: scanEndIp.value,
    port: scanPort.value,
  });
}

function openAddDialog(discovered?: DiscoveredCamera) {
  editingCameraId.value = null;
  formData.value = {
    name: discovered?.name || discovered?.address || "",
    address: discovered?.address || "",
    port: discovered?.port || 80,
    username: lastCredentials.value.username,
    password: lastCredentials.value.password,
  };
  addDialog.value = true;
}

function openEditDialog(cam: CameraInfo) {
  editingCameraId.value = cam.id;
  formData.value = {
    name: cam.name,
    address: cam.address,
    port: cam.onvif_port,
    username: cam.username,
    password: cam.password,
  };
  addDialog.value = true;
}

async function handleTestConnection() {
  isTesting.value = true;
  try {
    await store.testConnection(
      formData.value.address,
      formData.value.port,
      formData.value.username,
      formData.value.password,
    );
    testResultColor.value = "success";
    testResultMessage.value = "連線成功！帳號密碼驗證通過。";
  } catch (e) {
    testResultColor.value = "error";
    testResultMessage.value = `連線失敗：${e}`;
  } finally {
    isTesting.value = false;
    showTestResult.value = true;
  }
}

async function handleSaveCamera() {
  lastCredentials.value = {
    username: formData.value.username,
    password: formData.value.password,
  };
  if (editingCameraId.value) {
    const editedId = editingCameraId.value;
    posterCache.delete(editedId);
    await store.updateCamera(
      editedId,
      formData.value.name,
      formData.value.address,
      formData.value.port,
      formData.value.username,
      formData.value.password,
    );
    const updated = store.cameras.find((camera) => camera.id === editedId);
    if (updated) {
      selectedCamera.value = updated;
    }
    if (selectedCamera.value?.id === editedId) {
      previewPoster.value = null;
      void loadCachedSnapshot(editedId, true);
    }
  } else {
    await store.saveCamera(
      formData.value.name,
      formData.value.address,
      formData.value.port,
      formData.value.username,
      formData.value.password,
    );
  }
  addDialog.value = false;
}

async function handleDelete(id: string) {
  if (selectedCamera.value?.id === id) {
    selectedCamera.value = null;
    previewPoster.value = null;
  }
  posterCache.delete(id);
  await store.deleteCamera(id);
}

async function copyStreamUri() {
  if (store.currentStreamUri) {
    await navigator.clipboard.writeText(store.currentStreamUri);
    showCopySuccess.value = true;
  }
}

async function handleDiagnose() {
  isDiagnosing.value = true;
  try {
    diagnoseReport.value = await store.diagnoseCamera(
      formData.value.address,
      formData.value.port,
      formData.value.username,
      formData.value.password,
    );
    diagnoseDialog.value = true;
  } catch (e) {
    diagnoseReport.value = `診斷失敗：${e}`;
    diagnoseDialog.value = true;
  } finally {
    isDiagnosing.value = false;
  }
}

async function copyDiagnoseReport() {
  await navigator.clipboard.writeText(diagnoseReport.value);
  showCopySuccess.value = true;
}
</script>

<style scoped>
.home-container {
  height: calc(100vh - 96px);
  min-height: 680px;
}

.home-row {
  height: 100%;
}

.left-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.left-panel-card {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.left-panel-header {
  flex: 0 0 auto;
}

.tab-page {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.list-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.right-panel {
  display: flex;
  height: 100%;
}

.preview-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.preview-body {
  flex: 1;
  min-height: 0;
}

.preview-content {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.preview-content .preview-frame {
  flex: 1 1 auto;
  height: auto;
  min-height: 260px;
}

.preview-frame {
  background: #000;
  height: 100%;
  min-height: 320px;
  position: relative;
}

.preview-stage {
  overflow: hidden;
}

.preview-video {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  width: 100%;
  height: 100%;
  background: #000;
  position: relative;
  z-index: 1;
}

.preview-poster {
  position: absolute;
  inset: 0;
  z-index: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: #000;
  opacity: 0.5;
  filter: blur(1px) saturate(0.85);
  pointer-events: none;
}

.preview-poster-static {
  opacity: 0.9;
  filter: none;
}

.cx-static-overlay {
  position: absolute;
  inset: 0;
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  padding-bottom: 24px;
  background: linear-gradient(180deg, transparent 52%, rgba(2, 6, 12, 0.85));
  pointer-events: none;
}

.cx-static-badge {
  font-family: var(--cx-font-mono);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--cx-cyan);
  text-shadow: 0 0 12px rgba(34, 211, 238, 0.5);
}

.cx-static-hint {
  margin-top: 8px;
  font-size: 0.8rem;
  color: var(--cx-muted);
}

.preview-video-hidden {
  opacity: 0;
}

.preview-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.72);
}

.stream-select {
  min-width: 150px;
  max-width: 180px;
}

/* --- Cyberpunk HUD layer --- */

.cx-panel-strip {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 16px 0;
}

.cx-strip-meta {
  font-family: var(--cx-font-mono);
  font-size: 11px;
  letter-spacing: 0.12em;
  color: var(--cx-muted);
}

.cx-result-label {
  font-family: var(--cx-font-mono);
  font-size: 11px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--cx-muted);
}

.cx-result-label b {
  margin-left: 2px;
  font-weight: 600;
  color: var(--cx-cyan);
  text-shadow: 0 0 12px rgba(34, 211, 238, 0.5);
}

.preview-header {
  padding-top: 14px;
  padding-bottom: 12px;
  padding-right: 24px;
  align-content: center;
}

.preview-header :deep(.v-checkbox),
.preview-header :deep(.v-switch) {
  flex: 0 0 auto;
  min-width: max-content;
}

.preview-heading {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
  flex: 0 0 auto;
}

.preview-toolbar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
  margin-left: auto;
  min-width: 0;
}

.preview-toolbar :deep(.v-input) {
  flex: 0 0 auto;
}

.preview-heading-icon {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  flex: 0 0 auto;
  border: 1px solid var(--cx-line);
  background: rgba(34, 211, 238, 0.07);
  color: var(--cx-cyan);
  clip-path: polygon(
    0 0,
    100% 0,
    100% calc(100% - 8px),
    calc(100% - 8px) 100%,
    0 100%
  );
}

.preview-heading-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  line-height: 1.1;
}

.preview-heading-title {
  font-size: 1rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: var(--cx-text);
}

.cx-camera-chip.v-chip {
  background: rgba(34, 211, 238, 0.12) !important;
  color: var(--cx-cyan) !important;
  border: 1px solid var(--cx-line-strong);
}

.cx-camera-chip .v-chip__underlay {
  display: none;
}

.cx-live-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-left: 2px;
  padding: 3px 8px;
  border: 1px solid rgba(251, 113, 133, 0.45);
  background: rgba(251, 113, 133, 0.08);
  color: #fb7185;
  font-family: var(--cx-font-mono);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
  white-space: nowrap;
}

.cx-live-badge.is-pending {
  border-color: rgba(251, 191, 36, 0.45);
  background: rgba(251, 191, 36, 0.08);
  color: #fbbf24;
}

.cx-live-dot {
  width: 6px;
  height: 6px;
  flex: 0 0 auto;
  background: currentColor;
  box-shadow: 0 0 9px currentColor;
  animation: cx-pulse 1.6s ease-in-out infinite;
}

.cx-preview-frame {
  border: 1px solid rgba(34, 211, 238, 0.26);
  background:
    radial-gradient(
      ellipse at 50% 42%,
      rgba(34, 211, 238, 0.08),
      transparent 68%
    ),
    #02040a;
  box-shadow:
    inset 0 0 80px rgba(0, 0, 0, 0.85),
    0 0 34px rgba(34, 211, 238, 0.08);
}

.cx-preview-grid {
  position: absolute;
  inset: 0;
  z-index: 2;
  pointer-events: none;
  background-image:
    linear-gradient(rgba(34, 211, 238, 0.07) 1px, transparent 1px),
    linear-gradient(90deg, rgba(34, 211, 238, 0.07) 1px, transparent 1px);
  background-size: 44px 44px;
  opacity: 0.55;
  -webkit-mask-image: radial-gradient(
    ellipse 72% 72% at 50% 50%,
    transparent 42%,
    #000 100%
  );
  mask-image: radial-gradient(
    ellipse 72% 72% at 50% 50%,
    transparent 42%,
    #000 100%
  );
}

.cx-preview-scan {
  position: absolute;
  inset: 0;
  z-index: 3;
  pointer-events: none;
  background: repeating-linear-gradient(
    0deg,
    rgba(180, 240, 255, 0.035) 0px,
    rgba(180, 240, 255, 0.035) 1px,
    transparent 1px,
    transparent 4px
  );
  opacity: 0.55;
  overflow: hidden;
}

.cx-preview-scan::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  height: 20%;
  background: linear-gradient(
    180deg,
    transparent,
    rgba(34, 211, 238, 0.07),
    transparent
  );
  animation: cx-preview-sweep 7s linear infinite;
}

.cx-corner {
  position: absolute;
  z-index: 4;
  width: 24px;
  height: 24px;
  pointer-events: none;
  border: 0 solid var(--cx-cyan);
  opacity: 0.85;
  filter: drop-shadow(0 0 6px rgba(34, 211, 238, 0.6));
}

.cx-tl {
  top: 8px;
  left: 8px;
  border-top-width: 1px;
  border-left-width: 1px;
}

.cx-tr {
  top: 8px;
  right: 8px;
  border-top-width: 1px;
  border-right-width: 1px;
}

.cx-bl {
  bottom: 8px;
  left: 8px;
  border-bottom-width: 1px;
  border-left-width: 1px;
}

.cx-br {
  bottom: 8px;
  right: 8px;
  border-bottom-width: 1px;
  border-right-width: 1px;
}

.cx-loading-overlay {
  z-index: 6;
  background: rgba(2, 6, 12, 0.55);
  -webkit-backdrop-filter: blur(2px);
  backdrop-filter: blur(2px);
}

.cx-loading-title,
.cx-boot-title {
  margin-top: 14px;
  font-family: var(--cx-font-mono);
  font-size: 11px;
  letter-spacing: 0.22em;
  color: var(--cx-cyan);
  text-shadow: 0 0 14px rgba(34, 211, 238, 0.5);
}

.cx-boot {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.cx-boot-frame {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  min-height: 260px;
  overflow: hidden;
}

.cx-boot-poster {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: #000;
  opacity: 0.5;
  filter: blur(1px) saturate(0.85);
}

.cx-boot-overlay {
  position: relative;
  z-index: 1;
}

.cx-boot-ring {
  display: grid;
  place-items: center;
  width: 96px;
  height: 96px;
  border: 1px solid var(--cx-line);
  border-radius: 50%;
  background: radial-gradient(
    circle,
    rgba(34, 211, 238, 0.08),
    transparent 70%
  );
  box-shadow:
    0 0 40px rgba(34, 211, 238, 0.1),
    inset 0 0 30px rgba(34, 211, 238, 0.05);
}

.cx-stream-readout {
  border-left: 2px solid var(--cx-cyan) !important;
  background: rgba(34, 211, 238, 0.045) !important;
}

.cx-stream-tag {
  flex: 0 0 auto;
  margin-right: 10px;
  padding: 1px 5px;
  border: 1px solid var(--cx-line-strong);
  font-family: var(--cx-font-mono);
  font-size: 11px;
  letter-spacing: 0.1em;
  color: var(--cx-cyan);
}

.cx-stream-uri {
  min-width: 0;
  font-family: var(--cx-font-mono);
  color: var(--cx-muted);
}

.cx-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.cx-empty-ring {
  position: relative;
  display: grid;
  place-items: center;
  width: 104px;
  height: 104px;
  margin-bottom: 18px;
  border: 1px solid var(--cx-line-strong);
  border-radius: 50%;
  color: var(--cx-cyan) !important;
  background: radial-gradient(
    circle,
    rgba(34, 211, 238, 0.1),
    transparent 70%
  );
  box-shadow:
    0 0 44px rgba(34, 211, 238, 0.12),
    inset 0 0 32px rgba(34, 211, 238, 0.06);
}

.cx-empty-ring::before {
  content: "";
  position: absolute;
  inset: -11px;
  border: 1px dashed rgba(34, 211, 238, 0.24);
  border-radius: 50%;
  animation: cx-orbit 32s linear infinite;
}

.cx-empty-title {
  font-family: var(--cx-font-mono);
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.3em;
  color: var(--cx-text);
}

.cx-empty-hint {
  margin-top: 8px;
  font-size: 0.875rem;
  color: var(--cx-muted);
}

.cx-empty-meta {
  margin-top: 10px;
  font-family: var(--cx-font-mono);
  font-size: 11px;
  letter-spacing: 0.16em;
  color: var(--cx-muted);
  opacity: 0.7;
}

.cx-dialog {
  max-height: calc(100vh - 48px);
}

.cx-dialog :deep(.v-card-title) {
  padding-bottom: 12px;
  border-bottom: 1px solid var(--cx-line);
}

.cx-dialog :deep(.v-card-actions) {
  padding-top: 10px !important;
  padding-bottom: 14px !important;
  border-top: 1px solid var(--cx-line);
}

@keyframes cx-preview-sweep {
  0% {
    transform: translateY(-25%);
  }
  100% {
    transform: translateY(525%);
  }
}

@keyframes cx-orbit {
  to {
    transform: rotate(360deg);
  }
}

.v-theme--light .cx-live-badge {
  color: #be123c;
  border-color: rgba(190, 18, 60, 0.45);
}

.v-theme--light .cx-live-badge.is-pending {
  color: #b45309;
  border-color: rgba(180, 83, 9, 0.45);
}

@media (max-width: 1279px) {
  .home-container {
    height: auto;
    min-height: 0;
  }

  .home-row {
    height: auto;
  }

  .left-panel {
    height: auto;
    min-height: 0;
  }

  .left-panel-card {
    max-height: 58vh;
    min-height: 360px;
  }

  .right-panel {
    height: auto;
  }

  .preview-card {
    min-height: 420px;
  }

  .preview-frame {
    min-height: 260px;
  }
}
</style>
