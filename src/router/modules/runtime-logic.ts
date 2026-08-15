const runtimeLogic: AuthRoute.Route = {
  name: 'runtime-logic',
  path: '/runtime-logic',
  component: 'self',
  meta: {
    title: '运行逻辑',
    requiresAuth: true,
    singleLayout: 'basic',
    icon: 'mdi:sitemap-outline',
    order: 8
  }
};

export default runtimeLogic;
