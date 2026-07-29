import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { api } from '../composables/useApi';

export interface User {
  id: string;
  username: string;
  display_name: string;
  role: 'admin' | 'cashier' | 'kitchen' | 'waiter';
  is_active: boolean;
}

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null);
  const loading = ref(false);

  const isAuthenticated = computed(() => !!user.value);
  const role = computed(() => user.value?.role);

  async function fetchUser() {
    try {
      loading.value = true;
      const res = await api.get('/auth/me');
      user.value = res.data;
    } catch {
      user.value = null;
    } finally {
      loading.value = false;
    }
  }

  async function login(username: string, password: string) {
    const res = await api.post('/auth/login', { username, password });
    user.value = res.data.user;
    return res.data.user;
  }

  async function logout() {
    await api.post('/auth/logout');
    user.value = null;
  }

  return {
    user,
    loading,
    isAuthenticated,
    role,
    fetchUser,
    login,
    logout,
  };
});
