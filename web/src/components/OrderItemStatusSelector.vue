<template>
  <div class="status-btn-group">
    <button
      :class="['status-btn', 'btn-pending', { active: status === 'pending' }]"
      :disabled="disabled"
      @click="selectStatus('pending')"
    >
      Pending
    </button>
    <button
      :class="['status-btn', 'btn-preparing', { active: status === 'preparing' }]"
      :disabled="disabled"
      @click="selectStatus('preparing')"
    >
      Preparing 🔥
    </button>
    <button
      :class="['status-btn', 'btn-finished', { active: status === 'finished' }]"
      :disabled="disabled"
      @click="selectStatus('finished')"
    >
      Finished ✅
    </button>
    <button
      :class="['status-btn', 'btn-served', { active: status === 'served' }]"
      :disabled="disabled"
      @click="selectStatus('served')"
    >
      Served 🛎️
    </button>
    <button
      :class="['status-btn', 'btn-cancelled', { active: status === 'cancelled' }]"
      :disabled="disabled"
      @click="selectStatus('cancelled')"
    >
      Cancelled 🚫
    </button>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  status: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: 'change', status: string): void;
  (e: 'update:status', status: string): void;
}>();

function selectStatus(newStatus: string) {
  if (props.disabled || props.status === newStatus) return;
  emit('update:status', newStatus);
  emit('change', newStatus);
}
</script>

<style scoped>
.status-btn-group {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin-top: 0.5rem;
}

.status-btn {
  background: var(--bg-dark);
  color: var(--text-muted);
  border: 1px solid var(--border-color);
  padding: 0.3rem 0.55rem;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.status-btn:hover:not(:disabled) {
  border-color: var(--primary);
  color: white;
}

.status-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.status-btn.active {
  font-weight: 700;
  color: white;
}

.status-btn.btn-pending.active {
  background: #f59e0b;
  border-color: #f59e0b;
}

.status-btn.btn-preparing.active {
  background: #06b6d4;
  border-color: #06b6d4;
}

.status-btn.btn-finished.active {
  background: #10b981;
  border-color: #10b981;
}

.status-btn.btn-served.active {
  background: #6366f1;
  border-color: #6366f1;
}

.status-btn.btn-cancelled.active {
  background: #ef4444;
  border-color: #ef4444;
}
</style>
