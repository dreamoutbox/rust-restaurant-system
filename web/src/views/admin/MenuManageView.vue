<template>
  <AppLayout>
    <div class="menu-manage-wrapper">
      <div class="header-bar">
        <div>
          <h1>Food Menu & Category Management</h1>
          <p>Create dishes, set prices, toggle availability, and upload photos</p>
        </div>

        <div class="btn-group">
          <button class="btn-secondary" @click="showCategoryModal = true">+ Add Category</button>
          <button class="btn-primary" @click="showItemModal = true">+ Add Menu Item</button>
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
              <td class="price">${{ Number(item.price).toFixed(2) }}</td>
              <td>
                <button
                  :class="['toggle-btn', item.is_available ? 'active' : 'inactive']"
                  @click="toggleAvailability(item)"
                >
                  {{ item.is_available ? 'Available' : 'Sold Out' }}
                </button>
              </td>
              <td>
                <label class="btn-secondary sm-btn upload-lbl">
                  📷 Upload Image
                  <input type="file" accept="image/*" class="file-input" @change="uploadImage(item.id, $event)" />
                </label>
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

      <!-- Add Menu Item Modal -->
      <div v-if="showItemModal" class="modal-backdrop" @click.self="showItemModal = false">
        <div class="modal-card glass">
          <h3>Add Menu Item</h3>
          <form @submit.prevent="createMenuItem">
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
              <textarea v-model="itemForm.description" class="form-input" rows="3"></textarea>
            </div>

            <div class="form-group">
              <label>Price ($)</label>
              <input v-model.number="itemForm.price" type="number" step="0.01" class="form-input" required />
            </div>

            <div class="modal-actions">
              <button type="button" class="btn-secondary" @click="showItemModal = false">Cancel</button>
              <button type="submit" class="btn-primary">Create Item</button>
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

const menuItems = ref<any[]>([]);
const categories = ref<any[]>([]);
const loading = ref(true);

const showCategoryModal = ref(false);
const showItemModal = ref(false);

const catForm = ref({ name: '' });
const itemForm = ref({
  category_id: '',
  name: '',
  description: '',
  price: 10.0,
});

async function fetchData() {
  try {
    loading.value = true;
    const [catRes, menuRes] = await Promise.all([
      api.get('/categories'),
      api.get('/menu/all'),
    ]);
    categories.value = catRes.data;
    menuItems.value = menuRes.data;
    if (categories.value.length > 0) {
      itemForm.value.category_id = categories.value[0].id;
    }
  } catch (err) {
    console.error('Failed to fetch menu data', err);
  } finally {
    loading.value = false;
  }
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

async function createMenuItem() {
  try {
    await api.post('/menu', itemForm.value);
    showItemModal.value = false;
    itemForm.value = { category_id: categories.value[0]?.id || '', name: '', description: '', price: 10.0 };
    await fetchData();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to create menu item.');
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

async function uploadImage(itemId: string, event: Event) {
  const target = event.target as HTMLInputElement;
  if (!target.files || target.files.length === 0) return;

  const formData = new FormData();
  formData.append('image', target.files[0]);

  try {
    await api.post(`/menu/${itemId}/image`, formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
    await fetchData();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to upload image.');
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
  text-align: left;
}

.data-table th,
.data-table td {
  padding: 0.85rem 1rem;
  border-bottom: 1px solid var(--border-color);
}

.thumb {
  width: 44px;
  height: 44px;
  border-radius: 8px;
  background-size: cover;
  background-position: center;
  background-color: var(--bg-card-hover);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.25rem;
}

.sub-text {
  font-size: 0.8rem;
  color: var(--text-muted);
}

.category-badge {
  background: rgba(99, 102, 241, 0.15);
  color: #818cf8;
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
  padding: 0.3rem 0.7rem;
  font-size: 0.8rem;
  font-weight: 700;
  border-radius: 9999px;
}

.toggle-btn.active {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.toggle-btn.inactive {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

.upload-lbl {
  cursor: pointer;
  display: inline-block;
  padding: 0.35rem 0.75rem;
  font-size: 0.8rem;
}

.file-input {
  display: none;
}

.sm-btn {
  font-size: 0.75rem;
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-card {
  width: 100%;
  max-width: 460px;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  margin-top: 1rem;
}
</style>
