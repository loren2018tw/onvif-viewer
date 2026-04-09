<template>
  <v-container fluid>
    <v-row>
      <!-- Left panel: Camera list & discovery -->
      <v-col cols="12" md="4" lg="3">
        <v-card class="mb-4">
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

        <!-- Discovered cameras -->
        <v-card v-if="store.discoveredCameras.length > 0" class="mb-4">
          <v-card-title class="text-subtitle-1">
            <v-icon class="mr-2">mdi-access-point</v-icon>
            搜尋結果 ({{ store.discoveredCameras.length }})
          </v-card-title>
          <v-list density="compact">
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
        </v-card>

        <!-- Saved cameras -->
        <v-card>
          <v-card-title class="d-flex align-center">
            <v-icon class="mr-2">mdi-bookmark</v-icon>
            已儲存攝影機
            <v-spacer />
            <v-btn icon size="small" @click="openAddDialog()">
              <v-icon>mdi-plus</v-icon>
            </v-btn>
          </v-card-title>
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
                >{{ cam.address }}:{{ cam.onvif_port }}</v-list-item-subtitle
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
        </v-card>
      </v-col>

      <!-- Right panel: Preview -->
      <v-col cols="12" md="8" lg="9">
        <v-card class="fill-height" min-height="500">
          <v-card-title class="d-flex align-center">
            <v-icon class="mr-2">mdi-monitor</v-icon>
            即時預覽
            <v-spacer />
            <v-checkbox
              v-model="autoPreview"
              label="自動預覽"
              density="compact"
              hide-details
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
          <v-card-text
            class="d-flex justify-center align-center"
            style="min-height: 400px"
          >
            <div v-if="store.isLoadingStream" class="text-center">
              <v-progress-circular indeterminate color="primary" size="64" />
              <div class="mt-4 text-medium-emphasis">正在連線攝影機...</div>
            </div>
            <div
              v-else-if="store.isPreviewing && store.previewUrl"
              class="w-100"
            >
              <div class="d-flex justify-center" style="background: #000">
                <img
                  :src="store.previewUrl"
                  alt="攝影機預覽"
                  style="max-width: 100%; max-height: 70vh; object-fit: contain"
                  @error="onStreamError"
                />
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

    <!-- Diagnose result dialog -->
    <v-dialog v-model="diagnoseDialog" max-width="700">
      <v-card>
        <v-card-title>
          <v-icon class="mr-2">mdi-stethoscope</v-icon>
          ONVIF 診斷報告
        </v-card-title>
        <v-card-text>
          <v-sheet rounded class="text-body-2 pa-3" style="white-space: pre-wrap; word-break: break-all; font-family: monospace;" color="surface-variant"><pre class="ma-0">{{ diagnoseReport }}</pre></v-sheet>
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
import { ref, watch, onMounted } from "vue";
import { useCameraStore } from "@/stores/camera";
import type { CameraInfo, DiscoveredCamera } from "@/types/camera";

const store = useCameraStore();

const searchTab = ref("auto");
const scanStartIp = ref("192.168.1.1");
const scanEndIp = ref("192.168.1.254");
const scanPort = ref(80);
const selectedCamera = ref<CameraInfo | null>(null);
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

watch(
  () => store.error,
  (val) => {
    if (val) showError.value = true;
  },
);

onMounted(() => {
  store.loadCameras();
});

async function selectCamera(cam: CameraInfo) {
  selectedCamera.value = cam;
  if (autoPreview.value) {
    store.getStreamUri(cam.address, cam.onvif_port, cam.username, cam.password);
    await store.startPreview(
      cam.address,
      cam.onvif_port,
      cam.username,
      cam.password,
    );
  } else {
    store.stopPreview();
  }
}

async function handlePreview() {
  if (!selectedCamera.value) return;
  const cam = selectedCamera.value;
  // Get stream URI for display, and start MJPEG preview
  store.getStreamUri(cam.address, cam.onvif_port, cam.username, cam.password);
  await store.startPreview(
    cam.address,
    cam.onvif_port,
    cam.username,
    cam.password,
  );
}

async function handleStopPreview() {
  await store.stopPreview();
}

function onStreamError() {
  // MJPEG stream ended or errored — stop preview state
  if (store.isPreviewing) {
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
