<template>
  <v-app>
    <v-app-bar color="primary" density="compact">
      <v-app-bar-title>
        <v-icon class="mr-2">mdi-cctv</v-icon>
        ONVIF Camera Preview by Loren(loren.tw@gmail.com)
      </v-app-bar-title>
      <template #append>
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
import { computed } from "vue";
import { useTheme } from "vuetify";

const theme = useTheme();
const isDark = computed(() => theme.global.current.value.dark);

function toggleTheme() {
  theme.global.name.value = isDark.value ? "light" : "dark";
}
</script>
