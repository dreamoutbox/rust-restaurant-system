<template>
  <AppLayout>
    <div class="tables-manage-wrapper">
      <div class="header-bar">
        <div>
          <h1>Table Setup & Management</h1>
          <p>Configure physical restaurant dining tables</p>
        </div>

        <button class="btn-primary" @click="showAddModal = true">+ Add New Table</button>
      </div>

      <div v-if="loading" class="card">
        <p>Loading tables...</p>
      </div>

      <div v-else class="grid-cards">
        <div v-for="t in tables" :key="t.id" class="card table-manage-card">
          <div class="card-top">
            <h3>Table {{ t.table_number }}</h3>
            <span class="capacity">👥 {{ t.capacity || 4 }} Seats</span>
          </div>
          <p class="name">{{ t.name }}</p>

          <button class="btn-danger sm-btn" @click="deactivateTable(t.id)">Deactivate</button>
        </div>
      </div>

      <!-- Add Table Modal -->
      <div v-if="showAddModal" class="modal-backdrop" @click.self="showAddModal = false">
        <div class="modal-card glass">
          <h3>Add Dining Table</h3>

          <form @submit.prevent="createTable">
            <div class="form-group">
              <label>Table Number</label>
              <input v-model.number="form.table_number" type="number" class="form-input" required />
            </div>

            <div class="form-group">
              <label>Table Name / Area</label>
              <input v-model="form.name" type="text" class="form-input" placeholder="e.g. Window Seat A" required />
            </div>

            <div class="form-group">
              <label>Seating Capacity</label>
              <input v-model.number="form.capacity" type="number" class="form-input" required />
            </div>

            <div class="modal-actions">
              <button type="button" class="btn-secondary" @click="showAddModal = false">Cancel</button>
              <button type="submit" class="btn-primary">Add Table</button>
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

const tables = ref<any[]>([]);
const loading = ref(true);
const showAddModal = ref(false);

const form = ref({
  table_number: 1,
  name: '',
  capacity: 4,
});

async function fetchTables() {
  try {
    loading.value = true;
    const res = await api.get('/tables');
    tables.value = res.data;
  } catch (err) {
    console.error('Failed to fetch tables', err);
  } finally {
    loading.value = false;
  }
}

async function createTable() {
  try {
    await api.post('/tables', form.value);
    showAddModal.value = false;
    form.value = { table_number: tables.value.length + 1, name: '', capacity: 4 };
    await fetchTables();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to add table.');
  }
}

async function deactivateTable(id: string) {
  if (!confirm('Deactivate this table?')) return;
  try {
    await api.delete(`/tables/${id}`);
    await fetchTables();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to deactivate table.');
  }
}

onMounted(() => {
  fetchTables();
});
</script>

<style scoped>
.tables-manage-wrapper {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.header-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.table-manage-card {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.capacity {
  font-size: 0.85rem;
  color: var(--text-muted);
}

.name {
  font-size: 1.1rem;
  font-weight: 600;

}

.sm-btn {
  margin-top: 0.5rem;
  padding: 0.35rem 0.75rem;
  font-size: 0.8rem;
  align-self: flex-start;
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
  max-width: 440px;
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
