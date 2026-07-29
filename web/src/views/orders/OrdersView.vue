<template>
  <AppLayout>
    <div class="orders-wrapper">
      <div class="page-header">
        <div>
          <h2>Orders Overview</h2>
          <p class="subtitle">Real-time order tracking & history across all tables</p>
        </div>

        <div class="header-actions">
          <span :class="['live-indicator', { active: isConnected }]">
            <span class="pulse-dot"></span>
            {{ isConnected ? 'Live SSE Connected' : 'Connecting SSE...' }}
          </span>

          <button class="btn-secondary refresh-btn" @click="fetchOrders" :disabled="loading">
            🔄 Refresh
          </button>
        </div>
      </div>

      <!-- Filters Bar -->
      <div class="filters-bar glass">
        <div class="status-tabs">
          <button
            v-for="tab in statusTabs"
            :key="tab.value"
            :class="['tab-btn', { active: activeStatus === tab.value }]"
            @click="activeStatus = tab.value"
          >
            {{ tab.label }}
            <span class="count-pill">{{ getStatusCount(tab.value) }}</span>
          </button>
        </div>

        <div class="search-box">
          <span class="search-icon">🔍</span>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Filter by table # or order ID..."
            class="form-input search-input"
          />
        </div>
      </div>

      <!-- Loading State -->
      <div v-if="loading && orders.length === 0" class="loading-state card">
        <div class="spinner"></div>
        <p>Loading orders data...</p>
      </div>

      <!-- Empty State -->
      <div v-else-if="filteredOrders.length === 0" class="empty-state card">
        <span class="empty-icon">📋</span>
        <h3>No Orders Found</h3>
        <p v-if="searchQuery || activeStatus">Try clearing your search or status filter.</p>
        <p v-else>No active or historical order sessions registered yet.</p>
      </div>

      <!-- Orders List Grid -->
      <div v-else class="orders-grid">
        <div
          v-for="order in filteredOrders"
          :key="order.id"
          class="order-card card"
        >
          <div class="card-header">
            <div class="table-info">
              <span class="table-badge">Table {{ order.table_number }}</span>
              <span class="table-name">{{ order.table_name }}</span>
            </div>

            <StatusBadge :status="order.status" />
          </div>

          <div class="order-meta">
            <div class="meta-row">
              <span class="meta-label">Order ID:</span>
              <code class="order-id">{{ order.id.slice(0, 8) }}</code>
            </div>
            <div class="meta-row">
              <span class="meta-label">Opened At:</span>
              <span class="meta-val">{{ formatDate(order.opened_at) }}</span>
            </div>
            <div class="meta-row" v-if="order.closed_at">
              <span class="meta-label">Closed At:</span>
              <span class="meta-val">{{ formatDate(order.closed_at) }}</span>
            </div>
            <div class="meta-row" v-if="order.payment_method">
              <span class="meta-label">Payment Method:</span>
              <span class="payment-tag">{{ order.payment_method.toUpperCase() }}</span>
            </div>
          </div>

          <!-- Items Preview Table -->
          <div class="items-container">
            <div class="items-header">
              <span>Items ({{ order.items.length }})</span>
              <span class="items-total">Total: ${{ Number(order.total_amount).toFixed(2) }}</span>
            </div>

            <div v-if="order.items.length === 0" class="no-items">
              <p>No dishes ordered yet</p>
            </div>

            <div v-else class="items-list">
              <div
                v-for="item in order.items"
                :key="item.id"
                class="item-row"
              >
                <div class="item-left">
                  <span class="item-qty">{{ item.quantity }}x</span>
                  <span class="item-name">{{ item.menu_item_name }}</span>
                  <span v-if="item.note" class="item-note">({{ item.note }})</span>
                </div>

                <div class="item-right">
                  <span class="item-price">${{ (Number(item.unit_price) * item.quantity).toFixed(2) }}</span>

                  <!-- Status Selector for Staff -->
                  <div class="status-action" v-if="canManageItemStatus">
                    <select
                      :value="item.status"
                      @change="handleStatusChange(item.id, ($event.target as HTMLSelectElement).value)"
                      class="status-select"
                    >
                      <option value="pending">Pending</option>
                      <option value="preparing">Preparing</option>
                      <option value="finished">Finished</option>
                      <option value="served">Served</option>
                    </select>
                  </div>
                  <StatusBadge v-else :status="item.status" />
                </div>
              </div>
            </div>
          </div>

          <!-- Card Actions -->
          <div class="card-footer">
            <button
              v-if="['admin', 'cashier'].includes(authStore.role || '') && ['open', 'checkout_pending'].includes(order.status)"
              class="btn-primary action-btn"
              @click="router.push(`/cashier/table/${order.table_id}`)"
            >
              Go to Table POS 💳
            </button>
            <button
              class="btn-secondary action-btn"
              @click="selectedOrderModal = order"
            >
              Full Receipt View 📄
            </button>
          </div>
        </div>
      </div>

      <!-- Full Receipt Modal -->
      <div v-if="selectedOrderModal" class="modal-backdrop" @click.self="selectedOrderModal = null">
        <div class="receipt-modal card glass">
          <div class="modal-header">
            <h3>Receipt Summary — Order #{{ selectedOrderModal.id.slice(0, 8) }}</h3>
            <button class="close-btn" @click="selectedOrderModal = null">✕</button>
          </div>

          <div class="receipt-content">
            <div class="receipt-brand">
              <h2>🍽️ Rustaurant</h2>
              <p>Table {{ selectedOrderModal.table_number }} ({{ selectedOrderModal.table_name }})</p>
              <p class="receipt-date">{{ formatDate(selectedOrderModal.opened_at) }}</p>
            </div>

            <div class="divider"></div>

            <div class="receipt-items">
              <div v-for="item in selectedOrderModal.items" :key="item.id" class="receipt-row">
                <span>{{ item.quantity }}x {{ item.menu_item_name }}</span>
                <span>${{ (Number(item.unit_price) * item.quantity).toFixed(2) }}</span>
              </div>
            </div>

            <div class="divider"></div>

            <div class="receipt-summary">
              <div class="summary-row">
                <span>Status:</span>
                <StatusBadge :status="selectedOrderModal.status" />
              </div>
              <div class="summary-row" v-if="selectedOrderModal.payment_method">
                <span>Payment:</span>
                <span>{{ selectedOrderModal.payment_method.toUpperCase() }}</span>
              </div>
              <div class="summary-row total-line">
                <span>Grand Total:</span>
                <span class="total-price">${{ Number(selectedOrderModal.total_amount).toFixed(2) }}</span>
              </div>
            </div>
          </div>

          <div class="modal-footer">
            <button class="btn-secondary" @click="printReceipt">🖨️ Print Receipt</button>
            <button class="btn-primary" @click="selectedOrderModal = null">Close</button>
          </div>
        </div>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import AppLayout from '../../components/AppLayout.vue';
import StatusBadge from '../../components/StatusBadge.vue';
import { api } from '../../composables/useApi.ts';
import { useAuthStore } from '../../stores/auth.ts';
import { useSse } from '../../composables/useSse.ts';

interface OrderItem {
  id: string;
  order_id: string;
  table_number: number;
  menu_item_id: string;
  menu_item_name: string;
  quantity: number;
  unit_price: number;
  note: string | null;
  status: string;
  created_at: string;
}

interface OrderDetail {
  id: string;
  table_id: string;
  table_number: number;
  table_name: string;
  session_token: string;
  status: string;
  total_amount: number;
  payment_method: string | null;
  stripe_session_id: string | null;
  opened_at: string;
  closed_at: string | null;
  items: OrderItem[];
}

const authStore = useAuthStore();
const router = useRouter();

const orders = ref<OrderDetail[]>([]);
const loading = ref(true);
const activeStatus = ref<string>('all');
const searchQuery = ref('');
const selectedOrderModal = ref<OrderDetail | null>(null);

const canManageItemStatus = computed(() => {
  return ['admin', 'kitchen', 'waiter', 'cashier'].includes(authStore.role || '');
});

const statusTabs = [
  { label: 'All Orders', value: 'all' },
  { label: 'Open', value: 'open' },
  { label: 'Checkout Pending', value: 'checkout_pending' },
  { label: 'Paid', value: 'paid' },
  { label: 'Closed', value: 'closed' },
];

const { isConnected, connect } = useSse((event) => {
  console.log('SSE update received in OrdersView:', event);
  // Auto-refresh orders list on any SSE trigger
  fetchOrders();
});

const filteredOrders = computed(() => {
  return orders.value.filter((o) => {
    // Status Filter
    if (activeStatus.value !== 'all' && o.status !== activeStatus.value) {
      return false;
    }

    // Search Query
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase().trim();
      const matchTable = o.table_number.toString().includes(q) || o.table_name.toLowerCase().includes(q);
      const matchId = o.id.toLowerCase().includes(q);
      const matchItem = o.items.some((i) => i.menu_item_name.toLowerCase().includes(q));
      return matchTable || matchId || matchItem;
    }

    return true;
  });
});

function getStatusCount(status: string) {
  if (status === 'all') return orders.value.length;
  return orders.value.filter((o) => o.status === status).length;
}

function formatDate(isoStr: string | null) {
  if (!isoStr) return 'N/A';
  const d = new Date(isoStr);
  return d.toLocaleString([], {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

async function fetchOrders() {
  try {
    loading.value = true;
    const res = await api.get('/orders');
    orders.value = res.data;
  } catch (err) {
    console.error('Failed to fetch orders:', err);
  } finally {
    loading.value = false;
  }
}

async function handleStatusChange(itemId: string, newStatus: string) {
  try {
    await api.patch(`/order-items/${itemId}/status`, { status: newStatus });
    await fetchOrders();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to update item status');
  }
}

function printReceipt() {
  window.print();
}

onMounted(() => {
  fetchOrders();
  connect();
});
</script>

<style scoped>
.orders-wrapper {
  max-width: 1400px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.subtitle {
  color: var(--text-muted);
  font-size: 0.9rem;
  margin-top: 0.25rem;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.live-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
  font-weight: 600;
  padding: 0.3rem 0.75rem;
  border-radius: 9999px;
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.live-indicator.active {
  background: rgba(34, 197, 94, 0.15);
  color: #4ade80;
  border-color: rgba(34, 197, 94, 0.3);
}

.pulse-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0% { opacity: 0.4; }
  50% { opacity: 1; }
  100% { opacity: 0.4; }
}

.filters-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.25rem;
  margin-bottom: 1.5rem;
  border-radius: var(--radius);
  gap: 1rem;
  flex-wrap: wrap;
}

.status-tabs {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.tab-btn {
  background: var(--bg-card);
  color: var(--text-muted);
  border: 1px solid var(--border-color);
  padding: 0.45rem 0.9rem;
  border-radius: 9999px;
  font-size: 0.85rem;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.tab-btn.active {
  background: var(--primary);
  color: white;
  border-color: var(--primary);
}

.count-pill {
  background: rgba(255, 255, 255, 0.15);
  padding: 0.1rem 0.45rem;
  border-radius: 9999px;
  font-size: 0.75rem;
}

.search-box {
  position: relative;
  min-width: 280px;
}

.search-icon {
  position: absolute;
  left: 0.8rem;
  top: 50%;
  transform: translateY(-50%);
  font-size: 0.9rem;
  color: var(--text-muted);
}

.search-input {
  padding-left: 2.3rem;
  width: 100%;
}

.empty-state {
  text-align: center;
  padding: 4rem 2rem;
}

.empty-icon {
  font-size: 3rem;
  display: block;
  margin-bottom: 1rem;
}

.orders-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: 1.5rem;
}

.order-card {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.table-info {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.table-badge {
  background: var(--primary);
  color: white;
  font-weight: 700;
  font-size: 0.8rem;
  padding: 0.2rem 0.6rem;
  border-radius: 6px;
}

.table-name {
  font-weight: 600;
  font-size: 0.95rem;
}

.order-meta {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  padding: 0.75rem;
  margin-bottom: 1rem;
  font-size: 0.825rem;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.meta-row {
  display: flex;
  justify-content: space-between;
}

.meta-label {
  color: var(--text-muted);
}

.order-id {
  color: var(--accent);
  font-family: monospace;
}

.payment-tag {
  background: rgba(34, 197, 94, 0.2);
  color: #4ade80;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  font-weight: 700;
  font-size: 0.75rem;
}

.items-container {
  flex: 1;
  margin-bottom: 1rem;
}

.items-header {
  display: flex;
  justify-content: space-between;
  font-size: 0.85rem;
  font-weight: 700;
  color: var(--text-muted);
  margin-bottom: 0.6rem;
  padding-bottom: 0.4rem;
  border-bottom: 1px solid var(--border-color);
}

.items-total {
  color: var(--text-main);
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  max-height: 220px;
  overflow-y: auto;
}

.item-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.85rem;
  background: var(--bg-card-hover);
  padding: 0.4rem 0.6rem;
  border-radius: 6px;
}

.item-left {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  flex: 1;
  overflow: hidden;
}

.item-qty {
  font-weight: 700;
  color: var(--primary);
}

.item-name {
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-note {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-style: italic;
}

.item-right {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.item-price {
  font-weight: 600;
}

.status-select {
  background: var(--bg-dark);
  color: var(--text-main);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 0.75rem;
  padding: 0.15rem 0.4rem;
}

.card-footer {
  display: flex;
  gap: 0.75rem;
  margin-top: 0.5rem;
}

.action-btn {
  flex: 1;
  padding: 0.5rem;
  font-size: 0.85rem;
}

/* Modal */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: 1rem;
}

.receipt-modal {
  width: 100%;
  max-width: 460px;
  background: var(--bg-dark);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.25rem;
}

.receipt-content {
  background: #ffffff;
  color: #1e293b;
  padding: 1.5rem;
  border-radius: 8px;
  font-family: monospace;
  margin-bottom: 1.25rem;
}

.receipt-brand {
  text-align: center;
  margin-bottom: 1rem;
}

.receipt-brand h2 {
  font-size: 1.5rem;
  margin-bottom: 0.2rem;
}

.receipt-date {
  font-size: 0.75rem;
  color: #64748b;
}

.divider {
  border-bottom: 1px dashed #cbd5e1;
  margin: 1rem 0;
}

.receipt-items {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  font-size: 0.9rem;
}

.receipt-row {
  display: flex;
  justify-content: space-between;
}

.receipt-summary {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  font-size: 0.9rem;
}

.summary-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.total-line {
  font-size: 1.1rem;
  font-weight: 800;
  margin-top: 0.5rem;
}

.total-price {
  color: #0f172a;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
}
</style>
