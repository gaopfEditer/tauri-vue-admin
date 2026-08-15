const sampling: AuthRoute.Route = {
  name: 'sampling',
  path: '/sampling',
  component: 'basic',
  children: [
    {
      name: 'sampling_list',
      path: '/sampling/list',
      component: 'self',
      meta: {
        title: '采样编辑器',
        requiresAuth: true,
        icon: 'mdi:calendar-clock'
      }
    },
    {
      name: 'sampling_custom-fields',
      path: '/sampling/custom-fields',
      component: 'self',
      meta: {
        title: '自定义字段',
        requiresAuth: true,
        icon: 'mdi:form-textbox'
      }
    },
    {
      name: 'sampling_realtime',
      path: '/sampling/realtime',
      component: 'self',
      meta: {
        title: '实时信号',
        requiresAuth: true,
        icon: 'mdi:monitor-dashboard'
      }
    }
  ],
  meta: {
    title: '采样管理',
    icon: 'mdi:timer-outline',
    order: 4
  }
};

export default sampling;
