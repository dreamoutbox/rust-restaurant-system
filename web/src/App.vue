<template>
  <router-view />
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useAuthStore } from './stores/auth';

const authStore = useAuthStore();

onMounted(() => {
  if (!window.location.pathname.startsWith('/order/')) {
    authStore.fetchUser();
  }
});
</script>
