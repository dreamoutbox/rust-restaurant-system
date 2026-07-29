<template>
  <div class="customer-wrapper">
    <header class="customer-header glass">
      <div class="table-info">
        <span class="table-badge">Table {{ sessionData?.table_number }}</span>
        <h2>{{ sessionData?.table_name || 'Welcome' }}</h2>
      </div>

      <button class="cart-trigger btn-primary" @click="showCart = true">
        🛒 Cart ({{ totalCartCount }})
        <span class="cart-price" v-if="cartTotal > 0">${{ cartTotal.toFixed(2) }}</span>
      </button>
    </header>

    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <p>Loading table menu...</p>
    </div>

    <div v-else-if="error" class="error-state card">
      <h2>Session Expired or Invalid</h2>
      <p>{{ error }}</p>
    </div>

    <div v-else class="menu-content">
      <!-- Active Orders Summary if customer already ordered -->
      <section v-if="activeOrder && activeOrder.items.length > 0" class="active-orders-banner card">
        <div class="banner-header">
          <h3>Your Current Table Order</h3>
          <StatusBadge :status="activeOrder.status" />
        </div>
        <div class="ordered-items-list">
          <div v-for="item in activeOrder.items" :key="item.id" class="ordered-item-row">
            <span class="qty">{{ item.quantity }}x</span>
            <span class="name">{{ item.menu_item_name }}</span>
            <StatusBadge :status="item.status" />
            <span class="price">${{ (Number(item.unit_price) * item.quantity).toFixed(2) }}</span>
          </div>
        </div>
      </section>

      <!-- Category Filter Tabs -->
      <div class="category-tabs">
        <button
          :class="['tab-btn', { active: selectedCategory === null }]"
          @click="selectedCategory = null"
        >
          All Items
        </button>
        <button
          v-for="cat in categories"
          :key="cat"
          :class="['tab-btn', { active: selectedCategory === cat }]"
          @click="selectedCategory = cat"
        >
          {{ cat }}
        </button>
      </div>

      <!-- Menu Grid -->
      <div class="grid-cards">
        <div
          v-for="item in filteredMenuItems"
          :key="item.id"
          class="menu-card card"
        >
          <div class="item-img" :style="{ backgroundImage: item.image_path ? `url(${item.image_path})` : 'none' }">
            <span v-if="!item.image_path" class="img-placeholder">🍲</span>
          </div>

          <div class="item-details">
            <div class="item-header">
              <h4 class="item-title">{{ item.name }}</h4>
              <span class="item-price">${{ Number(item.price).toFixed(2) }}</span>
            </div>
            <p class="item-desc">{{ item.description || 'Delicious freshly prepared dish' }}</p>

            <div class="item-actions">
              <div class="qty-control" v-if="getItemCartQty(item.id) > 0">
                <button class="qty-btn" @click="updateCartQty(item.id, -1)">-</button>
                <span class="qty-num">{{ getItemCartQty(item.id) }}</span>
                <button class="qty-btn" @click="updateCartQty(item.id, 1)">+</button>
              </div>

              <button v-else class="btn-primary add-btn" @click="addToCart(item)">
                + Add to Order
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Cart Drawer Modal -->
    <div v-if="showCart" class="modal-backdrop" @click.self="showCart = false">
      <div class="cart-drawer glass">
        <div class="drawer-header">
          <h3>Your Order Cart</h3>
          <button class="close-btn" @click="showCart = false">✕</button>
        </div>

        <div v-if="cart.length === 0" class="empty-cart">
          <p>Your cart is empty. Pick something tasty!</p>
        </div>

        <div v-else class="cart-items">
          <div v-for="entry in cart" :key="entry.item.id" class="cart-item-row">
            <div class="cart-item-info">
              <h4>{{ entry.item.name }}</h4>
              <span class="unit-price">${{ Number(entry.item.price).toFixed(2) }}</span>
              <input
                v-model="entry.note"
                type="text"
                class="form-input note-input"
                placeholder="Add special instructions (e.g. no onions)"
              />
            </div>
            <div class="cart-item-actions">
              <button class="qty-btn" @click="updateCartQty(entry.item.id, -1)">-</button>
              <span>{{ entry.quantity }}</span>
              <button class="qty-btn" @click="updateCartQty(entry.item.id, 1)">+</button>
            </div>
          </div>
        </div>

        <div class="drawer-footer" v-if="cart.length > 0">
          <div class="total-row">
            <span>Subtotal</span>
            <span class="total-amount">${{ cartTotal.toFixed(2) }}</span>
          </div>

          <button class="btn-success submit-btn" :disabled="submitting" @click="submitOrder">
            {{ submitting ? 'Submitting Order...' : 'Send Order to Kitchen 🚀' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { api } from '../../composables/useApi.ts';
import StatusBadge from '../../components/StatusBadge.vue';

interface MenuItem {
  id: string;
  category_id: string;
  category_name: string;
  name: string;
  description: string;
  price: number;
  image_path: string | null;
}

interface CartEntry {
  item: MenuItem;
  quantity: number;
  note: string;
}

const route = useRoute();
const token = route.params.token as string;

const sessionData = ref<any>(null);
const menuItems = ref<MenuItem[]>([]);
const activeOrder = ref<any>(null);
const loading = ref(true);
const error = ref('');

const selectedCategory = ref<string | null>(null);
const cart = ref<CartEntry[]>([]);
const showCart = ref(false);
const submitting = ref(false);

const categories = computed(() => {
  const set = new Set<string>();
  menuItems.value.forEach((i) => set.add(i.category_name));
  return Array.from(set);
});

const filteredMenuItems = computed(() => {
  if (!selectedCategory.value) return menuItems.value;
  return menuItems.value.filter((i) => i.category_name === selectedCategory.value);
});

const totalCartCount = computed(() => {
  return cart.value.reduce((sum, entry) => sum + entry.quantity, 0);
});

const cartTotal = computed(() => {
  return cart.value.reduce((sum, entry) => sum + Number(entry.item.price) * entry.quantity, 0);
});

function getItemCartQty(itemId: string) {
  const found = cart.value.find((c) => c.item.id === itemId);
  return found ? found.quantity : 0;
}

function addToCart(item: MenuItem) {
  cart.value.push({ item, quantity: 1, note: '' });
}

function updateCartQty(itemId: string, delta: number) {
  const idx = cart.value.findIndex((c) => c.item.id === itemId);
  if (idx !== -1) {
    cart.value[idx].quantity += delta;
    if (cart.value[idx].quantity <= 0) {
      cart.value.splice(idx, 1);
    }
  }
}

async function fetchSessionAndMenu() {
  try {
    loading.value = true;
    const res = await api.get(`/order/${token}/menu`);
    sessionData.value = res.data;
    menuItems.value = res.data.menu;

    // Also fetch current order status if already placed items
    fetchOrderStatus();
  } catch (err: any) {
    error.value = err.response?.data?.error || 'Unable to load table menu.';
  } finally {
    loading.value = false;
  }
}

async function fetchOrderStatus() {
  try {
    const res = await api.get(`/order/${token}/status`);
    activeOrder.value = res.data;
  } catch {
    // Ignore error if no order placed yet
  }
}

async function submitOrder() {
  if (cart.value.length === 0) return;
  submitting.value = true;

  try {
    const items = cart.value.map((entry) => ({
      menu_item_id: entry.item.id,
      quantity: entry.quantity,
      note: entry.note || null,
    }));

    await api.post(`/order/${token}/items`, { items });

    cart.value = [];
    showCart.value = false;
    await fetchOrderStatus();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to submit order.');
  } finally {
    submitting.value = false;
  }
}

onMounted(() => {
  fetchSessionAndMenu();
  // Poll order status every 10 seconds for customer view
  setInterval(fetchOrderStatus, 10000);
});
</script>

<style scoped>
.customer-wrapper {
  max-width: 1000px;
  margin: 0 auto;
  padding: 1rem;
}

.customer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.5rem;
  margin-bottom: 1.5rem;
  border-radius: var(--radius);
}

.table-badge {
  background: var(--primary);
  color: white;
  font-size: 0.75rem;
  font-weight: 700;
  padding: 0.2rem 0.6rem;
  border-radius: 6px;
  text-transform: uppercase;
}

.table-info h2 {
  font-size: 1.4rem;
  margin-top: 0.2rem;
}

.cart-trigger {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.95rem;
}

.cart-price {
  background: rgba(255, 255, 255, 0.2);
  padding: 0.15rem 0.5rem;
  border-radius: 6px;
}

.active-orders-banner {
  margin-bottom: 1.5rem;
  background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  border: 1px solid rgba(99, 102, 241, 0.3);
}

.banner-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.ordered-items-list {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.ordered-item-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-size: 0.9rem;
  background: rgba(255, 255, 255, 0.03);
  padding: 0.5rem 0.8rem;
  border-radius: 8px;
}

.ordered-item-row .qty {
  font-weight: 700;
  color: var(--primary);
}

.ordered-item-row .name {
  flex: 1;
}

.ordered-item-row .price {
  font-weight: 600;
}

.category-tabs {
  display: flex;
  gap: 0.5rem;
  overflow-x: auto;
  padding-bottom: 0.5rem;
  margin-bottom: 1.25rem;
}

.tab-btn {
  background: var(--bg-card);
  color: var(--text-muted);
  border: 1px solid var(--border-color);
  padding: 0.5rem 1rem;
  border-radius: 9999px;
  font-size: 0.875rem;
  font-weight: 600;
  white-space: nowrap;
}

.tab-btn.active {
  background: var(--primary);
  color: white;
  border-color: var(--primary);
}

.menu-card {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.item-img {
  height: 150px;
  background-size: cover;
  background-position: center;
  background-color: #334155;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  margin-bottom: 1rem;
}

.img-placeholder {
  font-size: 3rem;
}

.item-details {
  display: flex;
  flex-direction: column;
  flex: 1;
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 0.4rem;
}

.item-title {
  font-size: 1.05rem;
  font-weight: 700;
}

.item-price {
  font-weight: 800;
  color: var(--accent);
}

.item-desc {
  font-size: 0.825rem;
  color: var(--text-muted);
  margin-bottom: 1rem;
  flex: 1;
}

.item-actions {
  margin-top: auto;
}

.add-btn {
  width: 100%;
  padding: 0.55rem;
  font-size: 0.9rem;
}

.qty-control {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-card-hover);
  border-radius: var(--radius);
  padding: 0.3rem 0.6rem;
}

.qty-btn {
  background: var(--primary);
  color: white;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  font-weight: 700;
}

.qty-num {
  font-weight: 700;
}

/* Cart Modal */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  justify-content: flex-end;
  z-index: 100;
}

.cart-drawer {
  width: 100%;
  max-width: 440px;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 1.5rem;
  background: var(--bg-dark);
}

.drawer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.close-btn {
  background: none;
  color: var(--text-muted);
  font-size: 1.25rem;
}

.cart-items {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.cart-item-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border-color);
}

.note-input {
  margin-top: 0.5rem;
  font-size: 0.8rem;
  padding: 0.4rem;
}

.drawer-footer {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid var(--border-color);
}

.total-row {
  display: flex;
  justify-content: space-between;
  font-size: 1.1rem;
  font-weight: 700;
  margin-bottom: 1rem;
}

.submit-btn {
  width: 100%;
  padding: 0.8rem;
  font-size: 1rem;
}
</style>
