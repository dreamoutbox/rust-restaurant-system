<template>
  <AppLayout>
    <div class="menu-manage-wrapper">
      <div class="header-bar">
        <div>
          <h1>Food Menu & Category Management</h1>
          <p>Create dishes, set prices in cents/dollars, toggle availability, and upload photos</p>
        </div>

        <div class="btn-group">
          <button class="btn-secondary" @click="openAddCategoryModal">+ Add Category</button>
          <button class="btn-primary" @click="openAddItemModal">+ Add Menu Item</button>
        </div>
      </div>

      <div v-if="loading" class="card">
        <p>Loading menu data...</p>
      </div>

      <div v-else class="menu-table-card card">
        <table class="data-table">
          <thead>
            <tr>
              <th>Image</th>
              <th>Name</th>
              <th>Category</th>
              <th>Price</th>
              <th>Available</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in menuItems" :key="item.id">
              <td>
                <div class="thumb" :style="{ backgroundImage: item.image_path ? `url(${item.image_path})` : 'none' }">
                  <span v-if="!item.image_path">🍲</span>
                </div>
              </td>
              <td class="font-bold">
                <div>{{ item.name }}</div>
                <div class="sub-text">{{ item.description }}</div>
              </td>
              <td>
                <span class="category-badge">{{ item.category_name }}</span>
              </td>
              <td class="price">${{ formatCents(item.price) }}</td>
              <td>
                <button
                  :class="['toggle-btn', item.is_available ? 'active' : 'inactive']"
                  @click="toggleAvailability(item)"
                >
                  {{ item.is_available ? 'Available' : 'Unavailable' }}
                </button>
              </td>
              <td class="action-cell">
                <button class="btn-secondary sm-btn" @click="openEditItemModal(item)">
                  ✏️ Edit
                </button>
                <button class="btn-danger sm-btn" @click="deleteMenuItem(item)">
                  🗑️ Delete
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Add Category Modal -->
      <div v-if="showCategoryModal" class="modal-backdrop" @click.self="showCategoryModal = false">
        <div class="modal-card glass">
          <h3>Add Category</h3>
          <form @submit.prevent="createCategory">
            <div class="form-group">
              <label>Category Name</label>
              <input v-model="catForm.name" type="text" class="form-input" required />
            </div>
            <div class="modal-actions">
              <button type="button" class="btn-secondary" @click="showCategoryModal = false">Cancel</button>
              <button type="submit" class="btn-primary">Save Category</button>
            </div>
          </form>
        </div>
      </div>

      <!-- Single Reusable Create / Edit Menu Item Modal -->
      <div v-if="showItemModal" class="modal-backdrop" @click.self="closeItemModal">
        <div class="modal-card glass">
          <h3>{{ editingItemId ? 'Edit Menu Item' : 'Add Menu Item' }}</h3>

          <form @submit.prevent="saveMenuItem">
            <div class="form-group">
              <label>Category</label>
              <select v-model="itemForm.category_id" class="form-input" required>
                <option v-for="c in categories" :key="c.id" :value="c.id">{{ c.name }}</option>
              </select>
            </div>

            <div class="form-group">
              <label>Dish Name</label>
              <input v-model="itemForm.name" type="text" class="form-input" required />
            </div>

            <div class="form-group">
              <label>Description</label>
              <textarea v-model="itemForm.description" class="form-input textarea" rows="2"></textarea>
            </div>

            <div class="form-group">
              <label>Price ($ USD)</label>
              <input
                v-model.number="itemForm.priceDollars"
                type="number"
                step="0.01"
                min="0"
                class="form-input"
                required
              />
            </div>

            <div class="form-group">
              <label>Dish Photo (Image Upload)</label>
              <input type="file" accept="image/*" class="form-input" @change="onFileSelected" />
              <p v-if="itemForm.existing_image_path" class="sub-text">Current: {{ itemForm.existing_image_path }}</p>
            </div>

            <div class="modal-actions">
              <button type="button" class="btn-secondary" @click="closeItemModal">Cancel</button>
              <button type="submit" class="btn-primary" :disabled="submitting">
                {{ submitting ? 'Saving...' : (editingItemId ? 'Update Item' : 'Create Item') }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import AppLayout from '../../components/AppLayout.vue';
import { api } from '../../composables/useApi.ts';

const categories = ref<any[]>([]);
const menuItems = ref<any[]>([]);
const loading = ref(true);

const showCategoryModal = ref(false);
const showItemModal = ref(false);
const editingItemId = ref<string | null>(null);
const submitting = ref(false);
const selectedFile = ref<File | null>(null);

const catForm = ref({ name: '' });
const itemForm = ref({
  category_id: '',
  name: '',
  description: '',
  priceDollars: 10.0,
  existing_image_path: null as string | null,
});

function formatCents(cents: number | string) {
  const c = typeof cents === 'string' ? parseInt(cents, 10) : cents;
  return ((c || 0) / 100).toFixed(2);
}

async function fetchData() {
  try {
    loading.value = true;
    const [catRes, menuRes] = await Promise.all([
      api.get('/categories'),
      api.get('/menu/all'),
    ]);
    categories.value = catRes.data;
    menuItems.value = menuRes.data;
  } catch (err) {
    console.error('Failed to fetch menu data', err);
  } finally {
    loading.value = false;
  }
}

function openAddCategoryModal() {
  catForm.value = { name: '' };
  showCategoryModal.value = true;
}

async function createCategory() {
  try {
    await api.post('/categories', catForm.value);
    showCategoryModal.value = false;
    catForm.value = { name: '' };
    await fetchData();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to create category.');
  }
}

function openAddItemModal() {
  editingItemId.value = null;
  selectedFile.value = null;
  itemForm.value = {
    category_id: categories.value[0]?.id || '',
    name: '',
    description: '',
    priceDollars: 10.0,
    existing_image_path: null,
  };
  showItemModal.value = true;
}

function openEditItemModal(item: any) {
  editingItemId.value = item.id;
  selectedFile.value = null;
  itemForm.value = {
    category_id: item.category_id,
    name: item.name,
    description: item.description || '',
    priceDollars: item.price / 100,
    existing_image_path: item.image_path,
  };
  showItemModal.value = true;
}

function closeItemModal() {
  showItemModal.value = false;
  editingItemId.value = null;
  selectedFile.value = null;
}

function onFileSelected(event: Event) {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    selectedFile.value = target.files[0];
  }
}

async function saveMenuItem() {
  submitting.value = true;
  try {
    const priceInCents = Math.round(itemForm.value.priceDollars * 100);
    const payload = {
      category_id: itemForm.value.category_id,
      name: itemForm.value.name,
      description: itemForm.value.description || null,
      price: priceInCents,
    };

    let itemId = editingItemId.value;

    if (itemId) {
      // Update item
      await api.put(`/menu/${itemId}`, payload);
    } else {
      // Create item
      const res = await api.post('/menu', payload);
      itemId = res.data.id;
    }

    // Upload image if selected
    if (selectedFile.value && itemId) {
      const formData = new FormData();
      formData.append('image', selectedFile.value);
      await api.post(`/menu/${itemId}/image`, formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      });
    }

    closeItemModal();
    await fetchData();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to save menu item.');
  } finally {
    submitting.value = false;
  }
}

async function toggleAvailability(item: any) {
  try {
    await api.put(`/menu/${item.id}`, { is_available: !item.is_available });
    await fetchData();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to toggle availability.');
  }
}

async function deleteMenuItem(item: any) {
  if (!confirm(`Are you sure you want to delete "${item.name}"?`)) return;
  try {
    await api.delete(`/menu/${item.id}`);
    await fetchData();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to delete menu item.');
  }
}

onMounted(() => {
  fetchData();
});
</script>

<style scoped>
.menu-manage-wrapper {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.header-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.btn-group {
  display: flex;
  gap: 0.75rem;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
}

.data-table th,
.data-table td {
  padding: 0.85rem 1rem;
  text-align: left;
  border-bottom: 1px solid var(--border-color);
}

.data-table th {
  color: var(--text-muted);
  font-size: 0.85rem;
  font-weight: 700;
  text-transform: uppercase;
}

.thumb {
  width: 44px;
  height: 44px;
  border-radius: 8px;
  background-size: cover;
  background-position: center;
  background-color: #334155;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.25rem;
}

.sub-text {
  font-size: 0.8rem;
  color: var(--text-muted);
  font-weight: normal;
}

.category-badge {
  background: var(--bg-card-hover);
  padding: 0.2rem 0.6rem;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
}

.price {
  font-weight: 800;
  color: var(--accent);
}

.toggle-btn {
  padding: 0.25rem 0.65rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 700;
}

.toggle-btn.active {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.toggle-btn.inactive {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.action-cell {
  display: flex;
  gap: 0.5rem;
}

.sm-btn {
  padding: 0.35rem 0.75rem;
  font-size: 0.8rem;
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-card {
  width: 100%;
  max-width: 480px;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.textarea {
  resize: vertical;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  margin-top: 1rem;
}
</style>
