import axios from 'axios';

export const api = axios.create({
  baseURL: '/api',
  withCredentials: true,
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response && error.response.status === 401) {
      // Token expired or unauthenticated
      if (window.location.pathname !== '/login' && !window.location.pathname.startsWith('/order/')) {
        window.location.href = '/login';
      }
    }
    return Promise.reject(error);
  }
);
