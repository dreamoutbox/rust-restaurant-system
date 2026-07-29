<template>
  <AppLayout>
    <div class="kitchen-wrapper">
      <div class="header-bar">
        <div>
          <h1>Kitchen Live Display Terminal</h1>
          <p>Real-time order item stream (Pending → Preparing → Finished)</p>
        </div>

        <div class="sse-status" :class="{ connected: isConnected }">
          <span class="pulse"></span>
          {{ isConnected ? 'Live SSE Connected' : 'Connecting Stream...' }}
        </div>
      </div>

      <div v-if="loading" class="card">
        <p>Loading order queue...</p>
      </div>

      <div v-else-if="kitchenItems.length === 0" class="empty-kitchen card">
        <span class="icon">👨‍🍳</span>
        <h3>Kitchen Queue is Clear!</h3>
        <p>No pending or preparing dishes at the moment.</p>
      </div>

      <!-- Grouped by Table -->
      <div v-else class="grid-cards">
        <div
          v-for="item in kitchenItems"
          :key="item.id"
          :class="['kitchen-card card', `status-${item.status}`]"
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
            ⚠️ Note: {{ item.note }}
          </div>

          <div class="time-stamp">
            Received: {{ formatTime(item.created_at) }}
          </div>

          <div class="action-buttons">
            <button
              v-if="item.status === 'pending'"
              class="btn-primary flex-1"
              @click="updateStatus(item.id, 'preparing')"
            >
              Start Cooking 🔥
            </button>

            <button
              v-if="item.status === 'preparing'"
              class="btn-success flex-1"
              @click="updateStatus(item.id, 'finished')"
            >
              Mark Finished ✅
            </button>
          </div>
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

interface KitchenItem {
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

const kitchenItems = ref<KitchenItem[]>([]);
const loading = ref(true);

const { isConnected, connect } = useSse((event) => {
  // Real-time SSE updates
  if (event.type === 'new_order_items' || event.type === 'item_status_changed') {
    fetchQueue();
  }
});

async function fetchQueue() {
  try {
    const res = await api.get('/orders');
    const allOrders = res.data;

    const items: KitchenItem[] = [];
    allOrders.forEach((ord: any) => {
      ord.items.forEach((item: any) => {
        // Kitchen cares about pending and preparing items
        if (['pending', 'preparing'].includes(item.status)) {
          items.push(item);
        }
      });
    });

    kitchenItems.value = items;
  } catch (err) {
    console.error('Error fetching kitchen queue', err);
  } finally {
    loading.value = false;
  }
}

async function updateStatus(itemId: string, status: string) {
  try {
    await api.patch(`/order-items/${itemId}/status`, { status });
    await fetchQueue();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to update item status.');
  }
}

function formatTime(iso: string) {
  return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

onMounted(() => {
  fetchQueue();
  connect();
});
</script>

<style scoped>
.kitchen-wrapper {
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

.empty-kitchen {
  text-align: center;
  padding: 4rem;
}

.empty-kitchen .icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.kitchen-card {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.kitchen-card.status-pending {
  border-left: 4px solid #f59e0b;
}

.kitchen-card.status-preparing {
  border-left: 4px solid #06b6d4;
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
  background: rgba(245, 158, 11, 0.15);
  color: #fbbf24;
  padding: 0.4rem 0.6rem;
  border-radius: 6px;
  font-size: 0.825rem;
  font-weight: 600;
}

.time-stamp {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.action-buttons {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.flex-1 {
  flex: 1;
}
</style>
