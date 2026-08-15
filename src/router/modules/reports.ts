const reports: AuthRoute.Route = {
  name: 'reports',
  path: '/reports',
  component: 'basic',
  children: [
    {
      name: 'reports_generator',
      path: '/reports/generator',
      component: 'self',
      meta: {
        title: '报表生成器',
        requiresAuth: true,
        icon: 'mdi:file-document-outline'
      }
    },
    {
      name: 'reports_sampling',
      path: '/reports/sampling',
      component: 'self',
      meta: {
        title: '采样报告',
        requiresAuth: true,
        icon: 'mdi:clipboard-text-outline'
      }
    },
    {
      name: 'reports_tags-catalog',
      path: '/reports/tags-catalog',
      component: 'self',
      meta: {
        title: 'Tags 目录',
        requiresAuth: true,
        icon: 'mdi:tag-multiple-outline'
      }
    },
    {
      name: 'reports_sda',
      path: '/reports/sda',
      component: 'self',
      meta: {
        title: '统计分析器',
        requiresAuth: true,
        icon: 'mdi:chart-bell-curve'
      }
    }
  ],
  meta: {
    title: '数据报表',
    icon: 'mdi:file-chart-outline',
    order: 9
  }
};

export default reports;
