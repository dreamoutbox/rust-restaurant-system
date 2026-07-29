<template>
  <AppLayout>
    <div class="pos-dashboard">
      <div class="header-bar">
        <div>
          <h1>Cashier POS — Tables Overview</h1>
          <p>Select a table to open session, generate QR, view orders, or process checkout</p>
        </div>

        <button class="btn-secondary" @click="fetchTables">🔄 Refresh Tables</button>
      </div>

      <div v-if="loading" class="loading-box card">
        <p>Loading tables...</p>
      </div>

      <div v-else class="grid-cards">
        <div
          v-for="table in tables"
          :key="table.id"
          :class="['table-card card', { occupied: table.active_order_id }]"
          @click="openTableDetail(table)"
        >
          <div class="card-top">
            <span class="table-num">Table {{ table.table_number }}</span>
            <StatusBadge
              :status="table.order_status || 'Available'"
            />
          </div>

          <h3 class="table-name">{{ table.name }}</h3>

          <div class="table-capacity" v-if="table.capacity">
            👥 Capacity: {{ table.capacity }} seats
          </div>

          <div class="card-footer">
            <span v-if="table.active_order_id" class="active-tag">Active Order Session</span>
            <span v-else class="vacant-tag">Available for Seating</span>
            <button class="btn-primary action-btn">Manage →</button>
          </div>
        </div>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import AppLayout from '../../components/AppLayout.vue';
import StatusBadge from '../../components/StatusBadge.vue';
import { api } from '../../composables/useApi.ts';

interface TableWithStatus {
  id: string;
  table_number: number;
  name: string;
  capacity?: number;
  is_active: boolean;
  active_order_id?: string;
  active_session_token?: string;
  order_status?: string;
}

const tables = ref<TableWithStatus[]>([]);
const loading = ref(true);
const router = useRouter();

async function fetchTables() {
  try {
    loading.value = true;
    const res = await api.get('/tables');
    tables.value = res.data.filter((t: any) => t.is_active);
  } catch (err: any) {
    console.error('Failed to fetch tables', err);
  } finally {
    loading.value = false;
  }
}

function openTableDetail(table: TableWithStatus) {
  router.push(`/cashier/table/${table.id}`);
}

onMounted(() => {
  fetchTables();
});
</script>

<style scoped>
.pos-dashboard {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.header-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-bar h1 {
  font-size: 1.6rem;
  font-weight: 800;
}

.table-card {
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  transition: transform 0.2s ease, border-color 0.2s ease;
}

.table-card:hover {
  transform: translateY(-2px);
  border-color: var(--primary);
}

.table-card.occupied {
  border-color: rgba(99, 102, 241, 0.4);
  background: linear-gradient(135deg, #1e293b 0%, #1e1b4b 100%);
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.table-num {
  font-weight: 800;
  font-size: 1.1rem;
  color: var(--primary);
}

.table-name {
  font-size: 1.2rem;
  font-weight: 700;
}

.table-capacity {
  font-size: 0.85rem;
  color: var(--text-muted);
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 0.5rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--border-color);
}

.active-tag {
  font-size: 0.75rem;
  font-weight: 700;
  color: #818cf8;
}

.vacant-tag {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.action-btn {
  padding: 0.35rem 0.75rem;
  font-size: 0.8rem;
}
</style>
