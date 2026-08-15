const audit: AuthRoute.Route = {
  name: 'audit',
  path: '/audit',
  component: 'self',
  meta: {
    title: '审计轨迹',
    requiresAuth: true,
    singleLayout: 'basic',
    icon: 'mdi:clipboard-text-clock-outline',
    order: 10
  }
};

export default audit;
