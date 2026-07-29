<template>
  <div class="layout-container">
    <header class="navbar glass">
      <div class="brand">
        <span class="logo-icon">🍽️</span>
        <span class="brand-title">Rustaurant</span>
        <span class="role-tag" v-if="authStore.user">{{ authStore.user.role }}</span>
      </div>

      <nav class="nav-links">
        <router-link
          v-if="['admin', 'cashier'].includes(authStore.role || '')"
          to="/cashier"
          class="nav-item"
        >
          POS & Tables
        </router-link>

        <router-link
          v-if="['admin', 'kitchen'].includes(authStore.role || '')"
          to="/kitchen"
          class="nav-item"
        >
          Kitchen
        </router-link>

        <router-link
          v-if="['admin', 'waiter'].includes(authStore.role || '')"
          to="/waiter"
          class="nav-item"
        >
          Waiter View
        </router-link>

        <router-link
          v-if="['admin', 'cashier', 'kitchen'].includes(authStore.role || '')"
          to="/admin/menu"
          class="nav-item"
        >
          Menu Management
        </router-link>

        <router-link
          v-if="authStore.role === 'admin'"
          to="/admin/tables"
          class="nav-item"
        >
          Table Setup
        </router-link>

        <router-link
          v-if="authStore.role === 'admin'"
          to="/admin/users"
          class="nav-item"
        >
          Staff Users
        </router-link>
      </nav>

      <div class="user-meta" v-if="authStore.user">
        <span class="user-name">{{ authStore.user.display_name }}</span>
        <button class="btn-secondary logout-btn" @click="handleLogout">Logout</button>
      </div>
    </header>

    <main class="main-content">
      <slot></slot>
    </main>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const authStore = useAuthStore();
const router = useRouter();

async function handleLogout() {
  await authStore.logout();
  router.push('/login');
}
</script>

<style scoped>
.layout-container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.navbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.85rem 2rem;
  margin: 1rem 1.5rem 0 1.5rem;
  z-index: 50;
}

.brand {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.logo-icon {
  font-size: 1.5rem;
}

.brand-title {
  font-size: 1.25rem;
  font-weight: 800;
  background: linear-gradient(135deg, #a5b4fc 0%, #6366f1 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.role-tag {
  background: rgba(99, 102, 241, 0.2);
  color: #a5b4fc;
  border: 1px solid rgba(99, 102, 241, 0.4);
  padding: 0.15rem 0.5rem;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
}

.nav-links {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.nav-item {
  color: var(--text-muted);
  font-weight: 600;
  font-size: 0.925rem;
  padding: 0.4rem 0.85rem;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.nav-item:hover,
.nav-item.router-link-active {
  color: white;
  background: var(--bg-card-hover);
}

.user-meta {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.user-name {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-main);
}

.logout-btn {
  padding: 0.4rem 0.85rem;
  font-size: 0.85rem;
}

.main-content {
  flex: 1;
  padding: 1.5rem 2rem;
  max-width: 1400px;
  width: 100%;
  margin: 0 auto;
}
</style>
