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
          <input
            v-model="username"
            type="text"
            class="form-input"
            placeholder="admin, cashier1, kitchen1, waiter1"
            required
          />
        </div>

        <div class="form-group">
          <label>Password</label>
          <input
            v-model="password"
            type="password"
            class="form-input"
            placeholder="Enter password"
            required
          />
        </div>

        <div v-if="errorMessage" class="error-box">
          {{ errorMessage }}
        </div>

        <button type="submit" class="btn-primary login-btn" :disabled="loading">
          {{ loading ? 'Signing in...' : 'Sign In' }}
        </button>
      </form>

      <div class="demo-hints">
        <h4>Demo Accounts (Password: admin / password):</h4>
        <ul>
          <li><strong>admin</strong> / admin</li>
          <li><strong>cashier1</strong> / password</li>
          <li><strong>kitchen1</strong> / password</li>
          <li><strong>waiter1</strong> / password</li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const username = ref('admin');
const password = ref('admin');
const errorMessage = ref('');
const loading = ref(false);

const authStore = useAuthStore();
const router = useRouter();

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
  max-width: 440px;
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

.demo-hints {
  margin-top: 2rem;
  padding-top: 1.25rem;
  border-top: 1px solid var(--border-color);
  font-size: 0.825rem;
  color: var(--text-muted);
}

.demo-hints h4 {
  color: white;
  margin-bottom: 0.5rem;
}

.demo-hints ul {
  list-style: none;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.4rem;
}
</style>
