const alarms: AuthRoute.Route = {
  name: 'alarms',
  path: '/alarms',
  component: 'self',
  meta: {
    title: '报警中心',
    requiresAuth: true,
    singleLayout: 'basic',
    icon: 'mdi:alarm-light-outline',
    order: 6
  }
};

export default alarms;
