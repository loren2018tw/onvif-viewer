<template>
  <v-app>
    <div class="cx-backdrop" aria-hidden="true"></div>
    <div class="cx-scanlines" aria-hidden="true"></div>

    <v-app-bar class="cx-appbar" height="64" flat>
      <v-app-bar-title>
        <div class="brand">
          <span class="brand-mark">
            <v-icon size="20">mdi-cctv</v-icon>
          </span>
          <span class="brand-text">
            <span class="brand-name">
              ONVIF<span class="brand-slash">//</span>VIEW
            </span>
            <span class="brand-sub">
              SURVEILLANCE CONSOLE · V{{ version }}
            </span>
          </span>
        </div>
      </v-app-bar-title>
      <template #append>
        <span class="cx-status mr-3" :class="`is-${systemStatus.tone}`">
          <span class="cx-status-dot"></span>
          {{ systemStatus.label }}
        </span>
        <span class="cx-credit mr-3">LOREN // loren.tw@gmail.com</span>
        <v-btn
          class="cx-icon-btn mr-3"
          icon
          size="small"
          :aria-label="isDark ? '切換至淺色模式' : '切換至深色模式'"
          :title="isDark ? '切換至淺色模式' : '切換至深色模式'"
          @click="toggleTheme"
        >
          <v-icon>
            {{ isDark ? "mdi-weather-sunny" : "mdi-weather-night" }}
          </v-icon>
        </v-btn>
      </template>
    </v-app-bar>
    <v-main>
      <router-view />
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useTheme } from "vuetify";
import { invoke } from "@tauri-apps/api/core";
import { useCameraStore } from "@/stores/camera";

const theme = useTheme();
const store = useCameraStore();
const isDark = computed(() => theme.global.current.value.dark);
const version = ref("");

const systemStatus = computed(() => {
  if (store.isPreviewing) {
    return { label: "STREAM.LIVE", tone: "live" };
  }
  if (store.isLoadingStream) {
    return { label: "STREAM.LINKING", tone: "pending" };
  }
  return { label: "SYS.READY", tone: "ready" };
});

onMounted(async () => {
  try {
    version.value = await invoke("get_app_version");
  } catch (error) {
    console.error("Failed to get version:", error);
    version.value = "0.0.0";
  }
});

function toggleTheme() {
  theme.global.name.value = isDark.value ? "light" : "dark";
}
</script>

<style scoped>
@media (max-width: 1100px) {
  .cx-credit {
    display: none;
  }
}

@media (max-width: 720px) {
  .cx-status {
    display: none;
  }

  .brand-sub {
    max-width: 200px;
  }
}

@media (max-width: 430px) {
  .brand-sub {
    display: none;
  }
}
</style>
