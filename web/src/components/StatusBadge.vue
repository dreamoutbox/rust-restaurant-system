<template>
  <span :class="['badge', badgeClass]">
    <span class="dot"></span>
    {{ label }}
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  status: string;
}>();

const badgeClass = computed(() => {
  switch (props.status) {
    case 'pending':
      return 'badge-pending';
    case 'preparing':
      return 'badge-preparing';
    case 'finished':
      return 'badge-finished';
    case 'served':
      return 'badge-served';
    case 'open':
      return 'badge-open';
    case 'paid':
      return 'badge-paid';
    default:
      return 'badge-served';
  }
});

const label = computed(() => {
  return props.status.replace('_', ' ');
});
</script>

<style scoped>
.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}
</style>
