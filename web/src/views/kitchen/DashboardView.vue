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

      <!-- Filter Tabs Bar -->
      <div class="filter-tabs glass">
        <button
          :class="['tab-btn', { active: activeFilter === 'cooking' }]"
          @click="activeFilter = 'cooking'"
        >
          Cooking Queue 🔥 ({{ countByStatus('pending') + countByStatus('preparing') }})
        </button>
        <button
          :class="['tab-btn', { active: activeFilter === 'pending' }]"
          @click="activeFilter = 'pending'"
        >
          Pending ({{ countByStatus('pending') }})
        </button>
        <button
          :class="['tab-btn', { active: activeFilter === 'preparing' }]"
          @click="activeFilter = 'preparing'"
        >
          Preparing 🔥 ({{ countByStatus('preparing') }})
        </button>
        <button
          :class="['tab-btn', { active: activeFilter === 'finished' }]"
          @click="activeFilter = 'finished'"
        >
          Finished ✅ ({{ countByStatus('finished') }})
        </button>
        <button
          :class="['tab-btn', { active: activeFilter === 'all' }]"
          @click="activeFilter = 'all'"
        >
          All Items ({{ allItems.length }})
        </button>
      </div>

      <div v-if="loading" class="card">
        <p>Loading order queue...</p>
      </div>

      <div v-else-if="filteredItems.length === 0" class="empty-kitchen card">
        <span class="icon">👨‍🍳</span>
        <h3>No Dishes in Selected Filter</h3>
        <p>Kitchen queue is clear for this filter category.</p>
      </div>

      <!-- Grouped by Table -->
      <div v-else class="grid-cards">
        <div
          v-for="item in filteredItems"
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

          <div class="status-btn-group">
            <button
              :class="['status-btn', { active: item.status === 'pending' }]"
              @click="updateStatus(item.id, 'pending')"
            >
              Pending
            </button>
            <button
              :class="['status-btn', { active: item.status === 'preparing' }]"
              @click="updateStatus(item.id, 'preparing')"
            >
              Preparing 🔥
            </button>
            <button
              :class="['status-btn', { active: item.status === 'finished' }]"
              @click="updateStatus(item.id, 'finished')"
            >
              Finished ✅
            </button>
            <button
              :class="['status-btn', { active: item.status === 'served' }]"
              @click="updateStatus(item.id, 'served')"
            >
              Served 🛎️
            </button>
          </div>
        </div>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
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

const allItems = ref<KitchenItem[]>([]);
const loading = ref(true);
const activeFilter = ref<string>('cooking');

const { isConnected, connect } = useSse((event) => {
  // Real-time SSE updates
  if (event.type === 'new_order_items' || event.type === 'item_status_changed') {
    fetchQueue();
  }
});

const filteredItems = computed(() => {
  if (activeFilter.value === 'cooking') {
    return allItems.value.filter((i) => i.status === 'pending' || i.status === 'preparing');
  }
  if (activeFilter.value === 'pending') {
    return allItems.value.filter((i) => i.status === 'pending');
  }
  if (activeFilter.value === 'preparing') {
    return allItems.value.filter((i) => i.status === 'preparing');
  }
  if (activeFilter.value === 'finished') {
    return allItems.value.filter((i) => i.status === 'finished');
  }
  return allItems.value;
});

function countByStatus(status: string) {
  return allItems.value.filter((i) => i.status === status).length;
}

async function fetchQueue() {
  try {
    const res = await api.get('/orders');
    const allOrders = res.data;

    const items: KitchenItem[] = [];
    allOrders.forEach((ord: any) => {
      ord.items.forEach((item: any) => {
        items.push(item);
      });
    });

    allItems.value = items;
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

.filter-tabs {
  display: flex;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
}

.tab-btn {
  background: var(--bg-card);
  color: var(--text-muted);
  border: 1px solid var(--border-color);
  padding: 0.45rem 0.9rem;
  border-radius: 9999px;
  font-size: 0.85rem;
  font-weight: 600;
}

.tab-btn.active {
  background: var(--primary);
  color: white;
  border-color: var(--primary);
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

.status-btn-group {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.4rem;
  margin-top: 0.5rem;
}

.status-btn {
  background: var(--bg-dark);
  color: var(--text-muted);
  border: 1px solid var(--border-color);
  padding: 0.35rem 0.5rem;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
}

.status-btn.active {
  background: var(--primary);
  color: white;
  border-color: var(--primary);
  font-weight: 700;
}

.flex-1 {
  flex: 1;
}
</style>
