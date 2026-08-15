const rtTrend: AuthRoute.Route = {
  name: 'rt-trend',
  path: '/rt-trend',
  component: 'self',
  meta: {
    title: '实时趋势',
    requiresAuth: true,
    singleLayout: 'basic',
    icon: 'mdi:chart-timeline-variant',
    order: 7
  }
};

export default rtTrend;
