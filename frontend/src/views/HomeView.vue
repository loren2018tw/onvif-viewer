<template>
  <v-container fluid class="home-container">
    <v-row class="home-row">
      <!-- Left panel: Camera list & discovery -->
      <v-col cols="12" lg="4" class="left-panel">
        <v-card class="mb-4 flex-shrink-0">
          <v-card-title class="d-flex align-center">
            <v-icon class="mr-2">mdi-magnify</v-icon>
            搜尋攝影機
          </v-card-title>
          <v-card-text>
            <v-tabs v-model="searchTab" density="compact" class="mb-3">
              <v-tab value="auto">自動搜尋</v-tab>
              <v-tab value="manual">手動掃描</v-tab>
            </v-tabs>

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
          </v-card-text>
        </v-card>

        <div class="list-split">
          <!-- Discovered cameras -->
          <v-card class="list-card">
            <v-card-title class="text-subtitle-1 d-flex align-center">
              <v-icon class="mr-2">mdi-access-point</v-icon>
              搜尋結果 ({{ store.discoveredCameras.length }})
            </v-card-title>
            <div class="list-scroll">
              <v-list
                v-if="store.discoveredCameras.length > 0"
                density="compact"
              >
                <v-list-item
                  v-for="(cam, idx) in store.discoveredCameras"
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
                尚無搜尋結果
              </v-card-text>
            </div>
          </v-card>

          <!-- Saved cameras -->
          <v-card class="list-card">
            <v-card-title class="d-flex align-center">
              <v-icon class="mr-2">mdi-bookmark</v-icon>
              已儲存攝影機
              <v-spacer />
              <v-btn icon size="small" @click="openAddDialog()">
                <v-icon>mdi-plus</v-icon>
              </v-btn>
            </v-card-title>
            <div class="list-scroll">
              <v-list v-if="store.cameras.length > 0" density="compact">
                <v-list-item
                  v-for="cam in store.cameras"
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
                尚無儲存的攝影機
              </v-card-text>
            </div>
          </v-card>
        </div>
      </v-col>

      <!-- Right panel: Preview -->
      <v-col cols="12" lg="8" class="right-panel">
        <v-card class="preview-card">
          <v-card-title
            class="d-flex align-center flex-wrap"
            style="row-gap: 8px; column-gap: 8px"
          >
            <v-icon class="mr-2">mdi-monitor</v-icon>
            即時預覽
            <v-spacer />
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
              class="mr-2 flex-grow-0"
            />
            <v-switch
              v-if="store.previewHasAudio"
              v-model="audioPlaybackEnabled"
              label="播放聲音"
              color="primary"
              density="compact"
              hide-details
              inset
              class="mr-2 flex-grow-0"
            />
            <v-chip
              v-if="selectedCamera"
              color="primary"
              size="small"
              class="mr-2"
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
              v-if="store.isPreviewing"
              color="error"
              size="small"
              class="ml-2"
              @click="handleStopPreview"
            >
              <v-icon class="mr-1">mdi-stop</v-icon>
              停止
            </v-btn>
          </v-card-title>
          <v-card-text class="preview-body d-flex justify-center align-center">
            <div
              v-if="store.isLoadingStream && !store.previewUrl"
              class="text-center"
            >
              <v-progress-circular indeterminate color="primary" size="64" />
              <div class="mt-4 text-medium-emphasis">正在連線攝影機...</div>
            </div>
            <div v-else-if="store.previewUrl" class="w-100 h-100">
              <div
                class="d-flex justify-center align-center preview-frame preview-stage"
              >
                <video
                  ref="previewVideo"
                  autoplay
                  playsinline
                  controls
                  class="preview-video"
                  :class="{ 'preview-video-hidden': !isPreviewVisible }"
                  @canplay="onPreviewReady"
                  @loadeddata="onPreviewReady"
                  @error="onPreviewPlaybackError"
                />
                <div
                  v-if="store.isLoadingStream"
                  class="preview-overlay text-center"
                >
                  <v-progress-circular
                    indeterminate
                    color="primary"
                    size="56"
                  />
                  <div class="mt-4 text-medium-emphasis">正在連線攝影機...</div>
                </div>
              </div>
              <div v-if="store.currentStreamUri" class="mt-3">
                <v-alert type="info" variant="tonal" density="compact">
                  <div class="d-flex align-center">
                    <code
                      class="text-caption flex-grow-1"
                      style="word-break: break-all"
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
            <div v-else class="text-center text-medium-emphasis">
              <v-icon size="120" color="grey">mdi-camera-off</v-icon>
              <div class="mt-4">請選擇一個攝影機並點選「預覽」</div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Error snackbar -->
    <v-snackbar v-model="showError" color="error" timeout="5000">
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
      <v-card>
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
      <v-card>
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
      <v-card>
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
const scanStartIp = ref("192.168.1.1");
const scanEndIp = ref("192.168.1.254");
const scanPort = ref(80);
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

let hls: Hls | null = null;
let attachedSessionId: string | null = null;

watch(
  () => store.error,
  (val) => {
    if (val) showError.value = true;
  },
);

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
      return;
    }

    attachedSessionId = sessionId;
    applyAudioPreference();

    if (video.canPlayType("application/vnd.apple.mpegurl")) {
      video.src = previewUrl;
    } else {
      hls = new Hls({
        enableWorker: true,
        lowLatencyMode: true,
      });
      hls.on(Hls.Events.ERROR, (_event, data) => {
        if (data.fatal) {
          onPreviewPlaybackError();
        }
      });
      hls.loadSource(previewUrl);
      hls.attachMedia(video);
    }

    video.play().catch(() => {
      // Waiting for canplay or a user gesture is acceptable here.
    });
  },
);

function destroyPreviewPlayer() {
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

  video.muted = !store.audioEnabled;
  if (store.audioEnabled) {
    video.play().catch(() => {
      // Browsers may still require an explicit gesture before unmuted playback.
    });
  }
}

async function selectCamera(cam: CameraInfo) {
  selectedCamera.value = cam;
  if (autoPreview.value) {
    await store.startPreview(
      cam.address,
      cam.onvif_port,
      cam.username,
      cam.password,
      selectedStreamType.value,
    );
  } else {
    store.stopPreview();
  }
}

async function handlePreview() {
  if (!selectedCamera.value) return;
  const cam = selectedCamera.value;
  await store.startPreview(
    cam.address,
    cam.onvif_port,
    cam.username,
    cam.password,
    selectedStreamType.value,
  );
}

async function handleStopPreview() {
  destroyPreviewPlayer();
  await store.stopPreview();
}

function onPreviewReady() {
  if (!store.previewSessionId || attachedSessionId !== store.previewSessionId) {
    return;
  }

  isPreviewVisible.value = true;
  store.markPreviewReady(store.previewSessionId);
  applyAudioPreference();
}

function onPreviewPlaybackError() {
  const activeSessionId = store.previewSessionId ?? undefined;
  destroyPreviewPlayer();
  store.markPreviewFailed(activeSessionId);
  if (activeSessionId) {
    store.stopPreview();
  }
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
    await store.updateCamera(
      editingCameraId.value,
      formData.value.name,
      formData.value.address,
      formData.value.port,
      formData.value.username,
      formData.value.password,
    );
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
  }
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

.list-split {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-rows: 1fr 1fr;
  gap: 16px;
}

.list-card {
  display: flex;
  flex-direction: column;
  min-height: 0;
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

@media (max-width: 1279px) {
  .home-container {
    height: auto;
    min-height: 0;
  }

  .left-panel,
  .right-panel {
    height: auto;
  }

  .list-split {
    height: auto;
    grid-template-rows: minmax(220px, 1fr) minmax(220px, 1fr);
  }

  .preview-card {
    min-height: 420px;
  }

  .preview-frame {
    min-height: 260px;
  }
}
</style>
