<template>
  <v-app>
    <v-app-bar color="primary" density="compact">
      <v-app-bar-title>
        <v-icon class="mr-2">mdi-cctv</v-icon>
        ONVIF Camera Preview
      </v-app-bar-title>
      <template #append>
        <div class="text-caption mr-4 grey-lighten-2">
          V{{ version }} by Loren(loren.tw@gmail.com)
        </div>
        <v-btn icon @click="toggleTheme">
          <v-icon>{{
            isDark ? "mdi-weather-sunny" : "mdi-weather-night"
          }}</v-icon>
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

const theme = useTheme();
const isDark = computed(() => theme.global.current.value.dark);
const version = ref("");

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
