import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/devices' },
    { path: '/devices', component: () => import('../pages/DevicesPage.vue') },
    { path: '/devices/new', component: () => import('../pages/DeviceForm.vue') },
    { path: '/devices/:id', component: () => import('../pages/DeviceForm.vue') },
    { path: '/groups', component: () => import('../pages/GroupsPage.vue') },
    { path: '/groups/new', component: () => import('../pages/GroupDetail.vue') },
    { path: '/groups/:id', component: () => import('../pages/GroupDetail.vue') },
    { path: '/access-points', component: () => import('../pages/APsPage.vue') },
    { path: '/access-points/new', component: () => import('../pages/APDetail.vue') },
    { path: '/access-points/:id', component: () => import('../pages/APDetail.vue') },
  ],
});

export default router;
