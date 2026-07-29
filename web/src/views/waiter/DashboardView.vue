<template>
  <AppLayout>
    <div class="waiter-wrapper">
      <div class="header-bar">
        <div>
          <h1>Waiter Terminal — Ready to Serve</h1>
          <p>Dishes finished by kitchen waiting for delivery</p>
        </div>

        <div class="sse-status" :class="{ connected: isConnected }">
          <span class="pulse"></span>
          {{ isConnected ? 'Live SSE Connected' : 'Connecting Stream...' }}
        </div>
      </div>

      <div v-if="loading" class="card">
        <p>Loading ready dishes...</p>
      </div>

      <div v-else-if="finishedItems.length === 0" class="empty-waiter card">
        <span class="icon">🛎️</span>
        <h3>No Dishes Ready to Serve</h3>
        <p>All cooked items have been delivered to tables.</p>
      </div>

      <div v-else class="grid-cards">
        <div
          v-for="item in finishedItems"
          :key="item.id"
          class="waiter-card card"
        >
          <div class="card-header">
            <span class="table-tag">Table {{ item.table_number }}</span>
            <StatusBadge :status="item.status" />
          </div>

          <div class="item-title">
            <span class="qty">{{ item.quantity }}x</span>
            <span class="name">{{ item.menu_item_name }}</span>
          </div>

          <div v-if="item.note" class="item-note">
            Note: {{ item.note }}
          </div>

          <button
            class="btn-success serve-btn"
            @click="markServed(item.id)"
          >
            Mark as Served 🛎️
          </button>
        </div>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import AppLayout from '../../components/AppLayout.vue';
import StatusBadge from '../../components/StatusBadge.vue';
import { api } from '../../composables/useApi.ts';
import { useSse } from '../../composables/useSse.ts';

interface WaiterItem {
  id: string;
  order_id: string;
  table_number: number;
  menu_item_id: string;
  menu_item_name: string;
  quantity: number;
  note: string | null;
  status: string;
  created_at: string;
}

const finishedItems = ref<WaiterItem[]>([]);
const loading = ref(true);

const { isConnected, connect } = useSse((event) => {
  if (event.type === 'item_status_changed' || event.type === 'new_order_items') {
    fetchFinishedQueue();
  }
});

async function fetchFinishedQueue() {
  try {
    const res = await api.get('/orders');
    const allOrders = res.data;

    const items: WaiterItem[] = [];
    allOrders.forEach((ord: any) => {
      ord.items.forEach((item: any) => {
        if (item.status === 'finished') {
          items.push(item);
        }
      });
    });

    finishedItems.value = items;
  } catch (err) {
    console.error('Error fetching waiter items', err);
  } finally {
    loading.value = false;
  }
}

async function markServed(itemId: string) {
  try {
    await api.patch(`/order-items/${itemId}/status`, { status: 'served' });
    await fetchFinishedQueue();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to mark as served.');
  }
}

onMounted(() => {
  fetchFinishedQueue();
  connect();
});
</script>

<style scoped>
.waiter-wrapper {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.header-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.sse-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
  padding: 0.4rem 0.85rem;
  border-radius: 9999px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  color: var(--text-muted);
}

.sse-status.connected {
  border-color: rgba(16, 185, 129, 0.4);
  color: #34d399;
}

.pulse {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
  animation: pulse-anim 1.5s infinite;
}

@keyframes pulse-anim {
  0% { opacity: 0.4; }
  50% { opacity: 1; }
  100% { opacity: 0.4; }
}

.empty-waiter {
  text-align: center;
  padding: 4rem;
}

.empty-waiter .icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.waiter-card {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  border-left: 4px solid #10b981;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.table-tag {
  font-size: 0.85rem;
  font-weight: 800;
  color: var(--primary);
  background: rgba(99, 102, 241, 0.15);
  padding: 0.2rem 0.6rem;
  border-radius: 6px;
}

.item-title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-size: 1.2rem;
  font-weight: 700;
}

.item-title .qty {
  color: var(--accent);
  font-weight: 800;
}

.item-note {
  font-size: 0.825rem;
  color: var(--text-muted);
}

.serve-btn {
  width: 100%;
  padding: 0.6rem;
  margin-top: 0.5rem;
}
</style>
