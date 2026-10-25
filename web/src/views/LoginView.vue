<template>
  <div class="login-wrapper">
    <div class="login-card glass">
      <div class="header">
        <div class="logo">🍽️</div>
        <h1>Rustaurant Staff Portal</h1>
        <p>Login to access POS, Kitchen, Waiter or Admin panel</p>
      </div>

      <form @submit.prevent="handleLogin">
        <div class="form-group">
          <label>Username</label>
          <input v-model="username" type="text" class="form-input" placeholder="admin, cashier1, kitchen1, waiter1"
            required />
        </div>

        <div class="form-group">
          <label>Password</label>
          <input v-model="password" type="password" class="form-input" placeholder="Enter password" required />
        </div>

        <div v-if="errorMessage" class="error-box">
          {{ errorMessage }}
        </div>

        <button type="submit" class="btn-primary login-btn" :disabled="loading">
          {{ loading ? 'Signing in...' : 'Sign In' }}
        </button>
      </form>

      <div class="demo-section">
        <div class="demo-title-row">
          <span class="demo-title">Demo Accounts</span>
          <span class="demo-hint">Click row to autofill</span>
        </div>
        <div class="demo-table-wrapper">
          <table class="demo-table">
            <thead>
              <tr>
                <th>Role</th>
                <th>Username</th>
                <th>Password</th>
                <th>Access Level</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="account in demoAccounts"
                :key="account.username"
                class="demo-row"
                :class="{ active: username === account.username }"
                @click="fillCredentials(account.username, account.password)"
              >
                <td>
                  <span class="role-badge" :class="account.roleClass">{{ account.role }}</span>
                </td>
                <td>
                  <code class="cred-code">{{ account.username }}</code>
                </td>
                <td>
                  <code class="cred-code">{{ account.password }}</code>
                </td>
                <td class="access-cell">{{ account.accessLevel }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';

interface DemoAccount {
  role: string;
  roleClass: string;
  username: string;
  password: string;
  accessLevel: string;
}

const demoAccounts: DemoAccount[] = [
  {
    role: 'Admin',
    roleClass: 'role-admin',
    username: 'admin',
    password: 'admin',
    accessLevel: 'Full system configuration & management',
  },
  {
    role: 'Cashier',
    roleClass: 'role-cashier',
    username: 'cashier1',
    password: 'password',
    accessLevel: 'POS checkout, receipts & table management',
  },
  {
    role: 'Kitchen',
    roleClass: 'role-kitchen',
    username: 'kitchen1',
    password: 'password',
    accessLevel: 'Kitchen queue & dish status updates',
  },
  {
    role: 'Waiter',
    roleClass: 'role-waiter',
    username: 'waiter1',
    password: 'password',
    accessLevel: 'Waiter dashboard & order status tracking',
  },
];

const username = ref('admin');
const password = ref('admin');
const errorMessage = ref('');
const loading = ref(false);

const authStore = useAuthStore();
const router = useRouter();

/**
 * Fills the login form with the provided demo account credentials.
 *
 * @param demoUsername - Username of the demo account.
 * @param demoPassword - Password of the demo account.
 *
 * @example
 * fillCredentials('cashier1', 'password');
 */
function fillCredentials(demoUsername: string, demoPassword: string) {
  username.value = demoUsername;
  password.value = demoPassword;
  errorMessage.value = '';
}

async function handleLogin() {
  errorMessage.value = '';
  loading.value = true;
  try {
    const user = await authStore.login(username.value, password.value);
    switch (user.role) {
      case 'cashier':
        router.push('/cashier');
        break;
      case 'kitchen':
        router.push('/kitchen');
        break;
      case 'waiter':
        router.push('/waiter');
        break;
      case 'admin':
        router.push('/cashier');
        break;
    }
  } catch (err: any) {
    errorMessage.value = err.response?.data?.error || 'Login failed. Please check credentials.';
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.login-wrapper {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1.5rem;
  background: radial-gradient(circle at top left, #1e1b4b 0%, #0f172a 100%);
}

.login-card {
  width: 100%;
  max-width: 520px;
  padding: 2.5rem;
}

.header {
  text-align: center;
  margin-bottom: 2rem;
}

.logo {
  font-size: 3rem;
  margin-bottom: 0.5rem;
}

.header h1 {
  font-size: 1.6rem;
  font-weight: 800;
  color: white;
}

.header p {
  color: var(--text-muted);
  font-size: 0.9rem;
  margin-top: 0.25rem;
}

.login-btn {
  width: 100%;
  padding: 0.8rem;
  font-size: 1rem;
  margin-top: 1rem;
}

.error-box {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.4);
  color: #fca5a5;
  padding: 0.75rem;
  border-radius: 8px;
  font-size: 0.875rem;
  margin-bottom: 1rem;
}

.demo-section {
  margin-top: 2rem;
  padding-top: 1.5rem;
  border-top: 1px solid var(--border-color);
}

.demo-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}

.demo-title {
  font-size: 0.8rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

.demo-hint {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.demo-table-wrapper {
  overflow-x: auto;
  border: 1px solid var(--border-color);
  border-radius: 8px;
}

.demo-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.78rem;
}

.demo-table th {
  text-align: left;
  padding: 0.5rem 0.6rem;
  background: rgba(255, 255, 255, 0.04);
  color: var(--text-muted);
  font-weight: 600;
  border-bottom: 1px solid var(--border-color);
  white-space: nowrap;
}

.demo-row {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.demo-row:hover {
  background: rgba(99, 102, 241, 0.12);
}

.demo-row.active {
  background: rgba(99, 102, 241, 0.2);
}

.demo-table td {
  padding: 0.5rem 0.6rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  vertical-align: middle;
}

.demo-table tr:last-child td {
  border-bottom: none;
}

.role-badge {
  display: inline-block;
  padding: 0.2rem 0.45rem;
  border-radius: 4px;
  font-weight: 600;
  font-size: 0.75rem;
  white-space: nowrap;
}

.role-admin {
  background: rgba(99, 102, 241, 0.25);
  color: #a5b4fc;
}

.role-cashier {
  background: rgba(16, 185, 129, 0.25);
  color: #6ee7b7;
}

.role-kitchen {
  background: rgba(245, 158, 11, 0.25);
  color: #fcd34d;
}

.role-waiter {
  background: rgba(6, 182, 212, 0.25);
  color: #67e8f9;
}

.cred-code {
  font-family: monospace;
  background: rgba(0, 0, 0, 0.35);
  padding: 0.15rem 0.35rem;
  border-radius: 4px;
  color: #e2e8f0;
  white-space: nowrap;
}

.access-cell {
  color: var(--text-muted);
  font-size: 0.75rem;
  line-height: 1.3;
}
</style>
