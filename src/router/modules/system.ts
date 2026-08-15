const system: AuthRoute.Route = {
  name: 'system',
  path: '/system',
  component: 'basic',
  children: [
    {
      name: 'system_backup',
      path: '/system/backup',
      component: 'self',
      meta: {
        title: '备份与恢复',
        requiresAuth: true,
        icon: 'mdi:backup-restore'
      }
    },
    {
      name: 'system_control',
      path: '/system/control',
      component: 'self',
      meta: {
        title: '系统控制',
        requiresAuth: true,
        icon: 'mdi:power'
      }
    },
    {
      name: 'system_language',
      path: '/system/language',
      component: 'self',
      meta: {
        title: '语言设置',
        requiresAuth: true,
        icon: 'mdi:translate'
      }
    },
    {
      name: 'system_reset-buffer',
      path: '/system/reset-buffer',
      component: 'self',
      meta: {
        title: '缓冲重置',
        requiresAuth: true,
        icon: 'mdi:database-refresh-outline'
      }
    },
    {
      name: 'system_facility',
      path: '/system/facility',
      component: 'self',
      meta: {
        title: '厂区组织',
        requiresAuth: true,
        icon: 'mdi:office-building-marker-outline'
      }
    },
    {
      name: 'system_license',
      path: '/system/license',
      component: 'self',
      meta: {
        title: '授权管理',
        requiresAuth: true,
        icon: 'mdi:license'
      }
    },
    {
      name: 'system_device-poll',
      path: '/system/device-poll',
      component: 'self',
      meta: {
        title: '设备轮询联调',
        requiresAuth: true,
        icon: 'mdi:lan-connect'
      }
    }
  ],
  meta: {
    title: '系统运维',
    icon: 'mdi:cog-outline',
    order: 11
  }
};

export default system;
