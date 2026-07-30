<template>
  <AppLayout>
    <div class="waiter-wrapper">
      <div class="header-bar">
        <div>
          <h1>Waiter Terminal — Table Service</h1>
          <p>Monitor order items and update status (Pending → Preparing → Finished → Served)</p>
        </div>

        <div class="sse-status" :class="{ connected: isConnected }">
          <span class="pulse"></span>
          {{ isConnected ? 'Live SSE Connected' : 'Connecting Stream...' }}
        </div>
      </div>

      <!-- Filter Tabs -->
      <div class="filter-tabs glass">
        <button
          :class="['tab-btn', { active: activeFilter === 'finished' }]"
          @click="activeFilter = 'finished'"
        >
          Ready to Serve 🛎️ ({{ countByStatus('finished') }})
        </button>
        <button
          :class="['tab-btn', { active: activeFilter === 'active' }]"
          @click="activeFilter = 'active'"
        >
          Cooking 🔥 ({{ countByStatus('preparing') + countByStatus('pending') }})
        </button>
        <button
          :class="['tab-btn', { active: activeFilter === 'served' }]"
          @click="activeFilter = 'served'"
        >
          Served ✅ ({{ countByStatus('served') }})
        </button>
        <button
          :class="['tab-btn', { active: activeFilter === 'cancelled' }]"
          @click="activeFilter = 'cancelled'"
        >
          Cancelled 🚫 ({{ countByStatus('cancelled') }})
        </button>
        <button
          :class="['tab-btn', { active: activeFilter === 'all' }]"
          @click="activeFilter = 'all'"
        >
          All Items ({{ allItems.length }})
        </button>
      </div>

      <div v-if="loading" class="card">
        <p>Loading dishes status...</p>
      </div>

      <div v-else-if="filteredItems.length === 0" class="empty-waiter card">
        <span class="icon">🛎️</span>
        <h3>No Dishes in Selected Filter</h3>
        <p>Queue is clear for this status category.</p>
      </div>

      <div v-else class="grid-cards">
        <div v-for="item in filteredItems" :key="item.id" class="waiter-card card">
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

          <!-- Reusable status control buttons -->
          <OrderItemStatusSelector :status="item.status" @change="updateItemStatus(item.id, $event)" />
        </div>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import AppLayout from '../../components/AppLayout.vue';
import StatusBadge from '../../components/StatusBadge.vue';
import OrderItemStatusSelector from '../../components/OrderItemStatusSelector.vue';
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

const allItems = ref<WaiterItem[]>([]);
const loading = ref(true);
const activeFilter = ref<string>('finished');

const { isConnected, connect } = useSse((event) => {
  if (event.type === 'item_status_changed' || event.type === 'new_order_items') {
    fetchOrdersQueue();
  }
});

const filteredItems = computed(() => {
  if (activeFilter.value === 'finished') {
    return allItems.value.filter((i) => i.status === 'finished');
  }
  if (activeFilter.value === 'active') {
    return allItems.value.filter((i) => i.status === 'pending' || i.status === 'preparing');
  }
  if (activeFilter.value === 'served') {
    return allItems.value.filter((i) => i.status === 'served');
  }
  if (activeFilter.value === 'cancelled') {
    return allItems.value.filter((i) => i.status === 'cancelled');
  }
  return allItems.value;
});

function countByStatus(status: string) {
  return allItems.value.filter((i) => i.status === status).length;
}

async function fetchOrdersQueue() {
  try {
    const res = await api.get('/orders');
    const allOrders = res.data;

    const items: WaiterItem[] = [];
    allOrders.forEach((ord: any) => {
      ord.items.forEach((item: any) => {
        items.push(item);
      });
    });

    allItems.value = items;
  } catch (err) {
    console.error('Error fetching waiter items', err);
  } finally {
    loading.value = false;
  }
}

async function updateItemStatus(itemId: string, status: string) {
  try {
    await api.patch(`/order-items/${itemId}/status`, { status });
    await fetchOrdersQueue();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to update item status.');
  }
}

onMounted(() => {
  fetchOrdersQueue();
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
</style>
