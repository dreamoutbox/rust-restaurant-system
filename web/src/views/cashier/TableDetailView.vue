<template>
  <AppLayout>
    <div class="table-detail-wrapper">
      <div class="top-nav">
        <button class="btn-secondary" @click="router.push('/cashier')">← Back to Tables</button>
        <span :class="['live-indicator', { active: isConnected }]">
          <span class="pulse-dot"></span>
          {{ isConnected ? 'Live SSE Connected' : 'Connecting SSE...' }}
        </span>
      </div>

      <div v-if="loading" class="card">
        <p>Loading table details...</p>
      </div>

      <div v-else-if="table" class="detail-grid">
        <!-- Left: Table Session & QR Code -->
        <div class="card session-card">
          <div class="session-header">
            <h2>Table {{ table.table_number }} — {{ table.name }}</h2>
            <StatusBadge :status="table.order_status || 'Vacant'" />
          </div>

          <!-- If Table Is Vacant -->
          <div v-if="!table.active_order_id" class="vacant-state">
            <p>Table is currently available. Click below to open a new dining session for customers.</p>
            <button class="btn-primary open-btn" :disabled="processing" @click="handleOpenTable">
              ✨ Open Table & Generate QR Code
            </button>
          </div>

          <!-- If Table Is Occupied -->
          <div v-else class="active-session-state">
            <div class="qr-container">
              <h3>Customer QR Code</h3>
              <p>Customers scan this to order dishes</p>

              <div class="qr-image-box">
                <img :src="`/api/tables/${table.id}/qr`" alt="Table QR Code" />
              </div>

              <div class="qr-url-box">
                <code>{{ customerOrderUrl }}</code>
              </div>
            </div>

            <div class="session-actions">
              <button class="btn-danger" :disabled="processing" @click="handleCloseTable">
                Close Table Session
              </button>
            </div>
          </div>
        </div>

        <!-- Right: Current Order Details & Checkout -->
        <div class="card order-card" v-if="table.active_order_id">
          <div class="order-header">
            <h3>Active Order #{{ activeOrderDetail?.id.slice(0, 8) }}</h3>
            <span class="total-badge" v-if="activeOrderDetail">
              Total: ${{ Number(activeOrderDetail.total_amount).toFixed(2) }}
            </span>
          </div>

          <div v-if="!activeOrderDetail" class="loading-order">
            <p>Fetching active order items...</p>
          </div>

          <div v-else class="order-body">
            <div class="items-list">
              <div v-for="item in activeOrderDetail.items" :key="item.id" class="order-item-row">
                <div class="item-main">
                  <span class="qty">{{ item.quantity }}x</span>
                  <span class="name">{{ item.menu_item_name }}</span>
                  <span class="price">${{ (Number(item.unit_price) * item.quantity).toFixed(2) }}</span>
                </div>
                <div class="item-sub">
                  <StatusBadge :status="item.status" />
                  <span v-if="item.note" class="note">Note: {{ item.note }}</span>
                </div>
              </div>

              <div v-if="activeOrderDetail.items.length === 0" class="no-items">
                <p>No dishes ordered yet. Customer is currently browsing menu.</p>
              </div>
            </div>

            <!-- Checkout Section -->
            <div class="checkout-section">
              <div class="payment-methods" v-if="activeOrderDetail.status !== 'paid'">
                <h4>Process Payment</h4>
                <div class="pay-btn-group">
                  <button class="btn-success pay-btn" :disabled="processing || activeOrderDetail.items.length === 0"
                    @click="handleManualPayment('cash')">
                    💵 Cash Payment
                  </button>
                  <button class="btn-primary pay-btn" :disabled="processing || activeOrderDetail.items.length === 0"
                    @click="handleManualPayment('card')">
                    💳 Card POS
                  </button>
                  <button class="btn-secondary pay-btn" :disabled="processing || activeOrderDetail.items.length === 0"
                    @click="handleStripePayment">
                    ⚡ Stripe Checkout
                  </button>
                </div>
              </div>

              <div v-else class="paid-success-banner">
                ✅ Order Paid via {{ activeOrderDetail.payment_method }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import AppLayout from '../../components/AppLayout.vue';
import StatusBadge from '../../components/StatusBadge.vue';
import { api } from '../../composables/useApi.ts';
import { useSse } from '../../composables/useSse.ts';

const route = useRoute();
const router = useRouter();
const tableId = route.params.id as string;

const table = ref<any>(null);
const activeOrderDetail = ref<any>(null);
const loading = ref(true);
const processing = ref(false);

const { isConnected, connect } = useSse((event) => {
  console.log('SSE update in TableDetailView:', event);
  fetchTableData();
});

const customerOrderUrl = computed(() => {
  if (!table.value?.active_session_token) return '';
  return `${window.location.origin}/order/${table.value.active_session_token}`;
});

async function fetchTableData() {
  try {
    loading.value = true;
    const res = await api.get('/tables');
    table.value = res.data.find((t: any) => t.id === tableId);

    if (table.value?.active_order_id) {
      const orderRes = await api.get(`/orders/${table.value.active_order_id}`);
      activeOrderDetail.value = orderRes.data;
    } else {
      activeOrderDetail.value = null;
    }
  } catch (err) {
    console.error('Error fetching table details', err);
  } finally {
    loading.value = false;
  }
}

async function handleOpenTable() {
  processing.value = true;
  try {
    await api.post(`/tables/${tableId}/open`);
    await fetchTableData();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to open table.');
  } finally {
    processing.value = false;
  }
}

async function handleCloseTable() {
  if (!confirm('Are you sure you want to close this table session?')) return;
  processing.value = true;
  try {
    await api.post(`/tables/${tableId}/close`);
    await fetchTableData();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to close table.');
  } finally {
    processing.value = false;
  }
}

async function handleManualPayment(method: string) {
  if (!activeOrderDetail.value) return;
  processing.value = true;
  try {
    // First calculate total/checkout
    await api.post(`/orders/${activeOrderDetail.value.id}/checkout`);
    // Then process manual payment
    await api.post(`/orders/${activeOrderDetail.value.id}/pay/manual`, {
      payment_method: method,
    });
    alert(`Payment recorded via ${method}!`);
    await fetchTableData();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to process payment.');
  } finally {
    processing.value = false;
  }
}

async function handleStripePayment() {
  if (!activeOrderDetail.value) return;
  processing.value = true;
  try {
    await api.post(`/orders/${activeOrderDetail.value.id}/checkout`);
    const res = await api.post(`/orders/${activeOrderDetail.value.id}/pay/stripe`);
    if (res.data.checkout_url) {
      window.location.href = res.data.checkout_url;
    }
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to initialize Stripe payment.');
  } finally {
    processing.value = false;
  }
}

onMounted(() => {
  fetchTableData();
  connect();
});
</script>

<style scoped>
.table-detail-wrapper {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.top-nav {
  display: flex;
  justify-content: space-between;
  align-items: center;
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
  0% {
    opacity: 0.4;
  }

  50% {
    opacity: 1;
  }

  100% {
    opacity: 0.4;
  }
}

.detail-grid {
  display: grid;
  grid-template-columns: 1fr 1.2fr;
  gap: 1.5rem;
}

.session-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.vacant-state {
  text-align: center;
  padding: 3rem 1.5rem;
}

.open-btn {
  margin-top: 1.5rem;
  padding: 0.8rem 1.5rem;
  font-size: 1rem;
}

.qr-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 0.75rem;
}

.qr-image-box {
  background: white;
  padding: 1rem;
  border-radius: 12px;
  margin: 1rem 0;
}

.qr-image-box img {
  width: 220px;
  height: 220px;
}

.qr-url-box {
  background: rgba(0, 0, 0, 0.4);
  padding: 0.5rem 1rem;
  border-radius: 8px;
  font-size: 0.8rem;
  word-break: break-all;
  max-width: 100%;
}

.session-actions {
  margin-top: 2rem;
  padding-top: 1rem;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
}

.order-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.25rem;
}

.total-badge {
  font-size: 1.1rem;
  font-weight: 800;
  color: var(--accent);
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  min-height: 200px;
}

.order-item-row {
  background: var(--bg-card-hover);
  padding: 0.8rem 1rem;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.item-main {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.item-main .qty {
  font-weight: 800;
  color: var(--primary);
}

.item-main .name {
  flex: 1;
  font-weight: 600;
}

.item-main .price {
  font-weight: 700;
}

.item-sub {
  display: flex;
  align-items: center;
  gap: 1rem;
  font-size: 0.8rem;
  color: var(--text-muted);
}

.checkout-section {
  margin-top: 2rem;
  padding-top: 1.25rem;
  border-top: 1px solid var(--border-color);
}

.pay-btn-group {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.75rem;
  margin-top: 0.75rem;
}

.pay-btn {
  padding: 0.7rem;
  font-size: 0.85rem;
}

.paid-success-banner {
  background: rgba(16, 185, 129, 0.2);
  border: 1px solid var(--accent);
  color: #34d399;
  padding: 1rem;
  border-radius: 8px;
  text-align: center;
  font-weight: 700;
}
</style>
