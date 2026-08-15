const systemView: AuthRoute.Route = {
  name: 'system-view',
  path: '/system-view',
  component: 'basic',
  meta: { title: 'system-view', icon: 'mdi:menu' },
  children: [
    {
      name: 'system-view_constant-page',
      path: '/system-view/constant-page',
      component: 'self',
      meta: { title: 'system-view_constant-page', icon: 'mdi:menu' }
    },
    {
      name: 'system-view_login',
      path: '/system-view/login',
      component: 'self',
      meta: { title: 'system-view_login', icon: 'mdi:menu' }
    },
    {
      name: 'system-view_no-permission',
      path: '/system-view/no-permission',
      component: 'self',
      meta: { title: 'system-view_no-permission', icon: 'mdi:menu' }
    },
    {
      name: 'system-view_not-found',
      path: '/system-view/not-found',
      component: 'self',
      meta: { title: 'system-view_not-found', icon: 'mdi:menu' }
    },
    {
      name: 'system-view_not-found-page',
      path: '/system-view/not-found-page',
      component: 'self',
      meta: { title: 'system-view_not-found-page', icon: 'mdi:menu' }
    },
    {
      name: 'system-view_service-error',
      path: '/system-view/service-error',
      component: 'self',
      meta: { title: 'system-view_service-error', icon: 'mdi:menu' }
    }
  ]
};

export default systemView;
