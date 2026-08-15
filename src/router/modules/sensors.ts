const sensors: AuthRoute.Route = {
  name: 'sensors',
  path: '/sensors',
  component: 'basic',
  children: [
    {
      name: 'sensors_particle',
      path: '/sensors/particle',
      component: 'self',
      meta: {
        title: '粒子传感器',
        requiresAuth: true,
        icon: 'mdi:circle-opacity'
      }
    },
    {
      name: 'sensors_biocapt',
      path: '/sensors/biocapt',
      component: 'self',
      meta: {
        title: '生物传感器',
        requiresAuth: true,
        icon: 'mdi:bacteria-outline'
      }
    },
    {
      name: 'sensors_analog',
      path: '/sensors/analog',
      component: 'self',
      meta: {
        title: '模拟量输入',
        requiresAuth: true,
        icon: 'mdi:sine-wave'
      }
    },
    {
      name: 'sensors_editor',
      path: '/sensors/editor',
      component: 'self',
      meta: {
        title: '传感器编辑',
        requiresAuth: true,
        icon: 'mdi:pencil-box-outline'
      }
    },
    {
      name: 'sensors_limits',
      path: '/sensors/limits',
      component: 'self',
      meta: {
        title: '限值编辑器',
        requiresAuth: true,
        icon: 'mdi:alert-octagon-outline'
      }
    }
  ],
  meta: {
    title: '设备传感器',
    icon: 'mdi:access-point',
    order: 5
  }
};

export default sensors;
