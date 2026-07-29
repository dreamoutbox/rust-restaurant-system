import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router';
import { useAuthStore } from '../stores/auth.ts';

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    redirect: '/login',
  },
  {
    path: '/login',
    name: 'login',
    component: () => import('../views/LoginView.vue'),
    meta: { public: true },
  },
  {
    path: '/order/:token',
    name: 'customer-order',
    component: () => import('../views/customer/OrderView.vue'),
    meta: { public: true },
  },
  {
    path: '/cashier',
    name: 'cashier-dashboard',
    component: () => import('../views/cashier/DashboardView.vue'),
    meta: { roles: ['admin', 'cashier'] },
  },
  {
    path: '/cashier/table/:id',
    name: 'cashier-table-detail',
    component: () => import('../views/cashier/TableDetailView.vue'),
    meta: { roles: ['admin', 'cashier'] },
  },
  {
    path: '/kitchen',
    name: 'kitchen-dashboard',
    component: () => import('../views/kitchen/DashboardView.vue'),
    meta: { roles: ['admin', 'kitchen'] },
  },
  {
    path: '/waiter',
    name: 'waiter-dashboard',
    component: () => import('../views/waiter/DashboardView.vue'),
    meta: { roles: ['admin', 'waiter'] },
  },
  {
    path: '/admin/users',
    name: 'admin-users',
    component: () => import('../views/admin/UserManageView.vue'),
    meta: { roles: ['admin'] },
  },
  {
    path: '/admin/tables',
    name: 'admin-tables',
    component: () => import('../views/admin/TableManageView.vue'),
    meta: { roles: ['admin'] },
  },
  {
    path: '/admin/menu',
    name: 'admin-menu',
    component: () => import('../views/admin/MenuManageView.vue'),
    meta: { roles: ['admin', 'cashier', 'kitchen'] },
  },
  {
    path: '/orders',
    name: 'orders-list',
    component: () => import('../views/orders/OrdersView.vue'),
    meta: { roles: ['admin', 'cashier', 'kitchen', 'waiter'] },
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach(async (to, _from, next) => {
  if (to.meta.public) {
    return next();
  }

  const authStore = useAuthStore();
  if (!authStore.user) {
    await authStore.fetchUser();
  }

  if (!authStore.isAuthenticated) {
    return next({ name: 'login' });
  }

  const allowedRoles = to.meta.roles as string[] | undefined;
  if (allowedRoles && !allowedRoles.includes(authStore.user!.role)) {
    // Redirect to their role default page
    switch (authStore.user!.role) {
      case 'cashier':
        return next({ name: 'cashier-dashboard' });
      case 'kitchen':
        return next({ name: 'kitchen-dashboard' });
      case 'waiter':
        return next({ name: 'waiter-dashboard' });
      case 'admin':
        return next({ name: 'cashier-dashboard' });
      default:
        return next({ name: 'login' });
    }
  }

  next();
});

export default router;
