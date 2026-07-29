<template>
  <AppLayout>
    <div class="tables-manage-wrapper">
      <div class="header-bar">
        <div>
          <h1>Table Setup & Management</h1>
          <p>Configure physical dining tables and manage active/deactivated states</p>
        </div>

        <button class="btn-primary" @click="openAddModal">+ Add New Table</button>
      </div>

      <!-- Filter Tabs -->
      <div class="filter-tabs">
        <button
          :class="['tab-btn', { active: filter === 'all' }]"
          @click="filter = 'all'"
        >
          All Tables ({{ tables.length }})
        </button>
        <button
          :class="['tab-btn', { active: filter === 'active' }]"
          @click="filter = 'active'"
        >
          Active ({{ activeCount }})
        </button>
        <button
          :class="['tab-btn', { active: filter === 'inactive' }]"
          @click="filter = 'inactive'"
        >
          Deactivated ({{ inactiveCount }})
        </button>
      </div>

      <div v-if="loading" class="card">
        <p>Loading tables...</p>
      </div>

      <div v-else-if="filteredTables.length === 0" class="card empty-card">
        <p>No tables found for this filter.</p>
      </div>

      <div v-else class="grid-cards">
        <div
          v-for="t in filteredTables"
          :key="t.id"
          :class="['card table-manage-card', { inactive: !t.is_active }]"
        >
          <div class="card-top">
            <div class="table-header-info">
              <h3>Table {{ t.table_number }}</h3>
              <span :class="['state-badge', t.is_active ? 'active' : 'inactive']">
                {{ t.is_active ? 'Active' : 'Deactivated' }}
              </span>
            </div>
            <span class="capacity">👥 {{ t.capacity || 4 }} Seats</span>
          </div>
          <p class="name">{{ t.name }}</p>

          <div class="card-actions">
            <button class="btn-secondary sm-btn" @click="openEditModal(t)">
              ✏️ Edit
            </button>
            <button
              v-if="t.is_active"
              class="btn-danger sm-btn"
              @click="toggleTableActive(t, false)"
            >
              Deactivate
            </button>
            <button
              v-else
              class="btn-success sm-btn"
              @click="toggleTableActive(t, true)"
            >
              ✨ Reactivate
            </button>
          </div>
        </div>
      </div>

      <!-- Add / Edit Table Modal -->
      <div v-if="showModal" class="modal-backdrop" @click.self="showModal = false">
        <div class="modal-card glass">
          <h3>{{ editingTableId ? 'Edit Dining Table' : 'Add Dining Table' }}</h3>

          <form @submit.prevent="saveTable">
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
              <button type="button" class="btn-secondary" @click="showModal = false">Cancel</button>
              <button type="submit" class="btn-primary" :disabled="submitting">
                {{ submitting ? 'Saving...' : (editingTableId ? 'Update Table' : 'Add Table') }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import AppLayout from '../../components/AppLayout.vue';
import { api } from '../../composables/useApi.ts';

const tables = ref<any[]>([]);
const loading = ref(true);
const showModal = ref(false);
const editingTableId = ref<string | null>(null);
const submitting = ref(false);
const filter = ref<'all' | 'active' | 'inactive'>('all');

const form = ref({
  table_number: 1,
  name: '',
  capacity: 4,
});

const activeCount = computed(() => tables.value.filter((t) => t.is_active).length);
const inactiveCount = computed(() => tables.value.filter((t) => !t.is_active).length);

const filteredTables = computed(() => {
  if (filter.value === 'active') return tables.value.filter((t) => t.is_active);
  if (filter.value === 'inactive') return tables.value.filter((t) => !t.is_active);
  return tables.value;
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

function openAddModal() {
  editingTableId.value = null;
  form.value = {
    table_number: tables.value.length + 1,
    name: '',
    capacity: 4,
  };
  showModal.value = true;
}

function openEditModal(t: any) {
  editingTableId.value = t.id;
  form.value = {
    table_number: t.table_number,
    name: t.name,
    capacity: t.capacity || 4,
  };
  showModal.value = true;
}

async function saveTable() {
  submitting.value = true;
  try {
    if (editingTableId.value) {
      await api.put(`/tables/${editingTableId.value}`, form.value);
    } else {
      await api.post('/tables', form.value);
    }
    showModal.value = false;
    await fetchTables();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to save table.');
  } finally {
    submitting.value = false;
  }
}

async function toggleTableActive(table: any, is_active: boolean) {
  const actionName = is_active ? 'Reactivate' : 'Deactivate';
  if (!confirm(`${actionName} Table ${table.table_number}?`)) return;

  try {
    if (is_active) {
      await api.put(`/tables/${table.id}`, { is_active: true });
    } else {
      await api.delete(`/tables/${table.id}`);
    }
    await fetchTables();
  } catch (err: any) {
    alert(err.response?.data?.error || `Failed to ${actionName.toLowerCase()} table.`);
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

.filter-tabs {
  display: flex;
  gap: 0.5rem;
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

.empty-card {
  text-align: center;
  padding: 3rem 1.5rem;
  color: var(--text-muted);
}

.table-manage-card {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  transition: opacity 0.2s ease, border-color 0.2s ease;
}

.table-manage-card.inactive {
  opacity: 0.6;
  border-style: dashed;
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.table-header-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.state-badge {
  font-size: 0.7rem;
  font-weight: 700;
  padding: 0.1rem 0.45rem;
  border-radius: 4px;
  text-transform: uppercase;
}

.state-badge.active {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.state-badge.inactive {
  background: rgba(148, 163, 184, 0.2);
  color: #94a3b8;
}

.capacity {
  font-size: 0.85rem;
  color: var(--text-muted);
}

.name {
  font-size: 1.1rem;
  font-weight: 600;
}

.card-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.sm-btn {
  padding: 0.35rem 0.75rem;
  font-size: 0.8rem;
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
