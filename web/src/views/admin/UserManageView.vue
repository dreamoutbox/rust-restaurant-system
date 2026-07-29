<template>
  <AppLayout>
    <div class="users-wrapper">
      <div class="header-bar">
        <div>
          <h1>Staff User Management</h1>
          <p>Create and manage accounts for Admin, Cashier, Kitchen, and Waiter staff</p>
        </div>

        <button class="btn-primary" @click="showAddModal = true">+ Add New User</button>
      </div>

      <div v-if="loading" class="card">
        <p>Loading users...</p>
      </div>

      <div v-else class="card table-container">
        <table class="data-table">
          <thead>
            <tr>
              <th>Username</th>
              <th>Display Name</th>
              <th>Role</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="u in users" :key="u.id">
              <td class="font-bold">{{ u.username }}</td>
              <td>{{ u.display_name }}</td>
              <td>
                <span class="role-badge">{{ u.role }}</span>
              </td>
              <td>
                <span :class="['status-dot', u.is_active ? 'active' : 'inactive']"></span>
                {{ u.is_active ? 'Active' : 'Inactive' }}
              </td>
              <td>
                <button class="btn-secondary sm-btn" @click="deactivateUser(u.id)" v-if="u.is_active">
                  Deactivate
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Add User Modal -->
      <div v-if="showAddModal" class="modal-backdrop" @click.self="showAddModal = false">
        <div class="modal-card glass">
          <h3>Create New Staff Account</h3>

          <form @submit.prevent="createUser">
            <div class="form-group">
              <label>Username</label>
              <input v-model="form.username" type="text" class="form-input" required />
            </div>

            <div class="form-group">
              <label>Display Name</label>
              <input v-model="form.display_name" type="text" class="form-input" required />
            </div>

            <div class="form-group">
              <label>Password</label>
              <input v-model="form.password" type="password" class="form-input" required />
            </div>

            <div class="form-group">
              <label>Role</label>
              <select v-model="form.role" class="form-input" required>
                <option value="admin">Admin</option>
                <option value="cashier">Cashier</option>
                <option value="kitchen">Kitchen</option>
                <option value="waiter">Waiter</option>
              </select>
            </div>

            <div class="modal-actions">
              <button type="button" class="btn-secondary" @click="showAddModal = false">Cancel</button>
              <button type="submit" class="btn-primary">Create User</button>
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

const users = ref<any[]>([]);
const loading = ref(true);
const showAddModal = ref(false);

const form = ref({
  username: '',
  display_name: '',
  password: '',
  role: 'cashier',
});

async function fetchUsers() {
  try {
    loading.value = true;
    const res = await api.get('/users');
    users.value = res.data;
  } catch (err) {
    console.error('Failed to fetch users', err);
  } finally {
    loading.value = false;
  }
}

async function createUser() {
  try {
    await api.post('/users', form.value);
    showAddModal.value = false;
    form.value = { username: '', display_name: '', password: '', role: 'cashier' };
    await fetchUsers();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to create user.');
  }
}

async function deactivateUser(id: string) {
  if (!confirm('Deactivate this user account?')) return;
  try {
    await api.delete(`/users/${id}`);
    await fetchUsers();
  } catch (err: any) {
    alert(err.response?.data?.error || 'Failed to deactivate user.');
  }
}

onMounted(() => {
  fetchUsers();
});
</script>

<style scoped>
.users-wrapper {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.header-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
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

.data-table th {
  color: var(--text-muted);
  font-size: 0.85rem;
  text-transform: uppercase;
}

.role-badge {
  background: var(--bg-card-hover);
  padding: 0.2rem 0.6rem;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 700;
  text-transform: uppercase;
}

.status-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 0.4rem;
}

.status-dot.active {
  background: var(--accent);
}

.status-dot.inactive {
  background: var(--danger);
}

.sm-btn {
  padding: 0.3rem 0.6rem;
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
