module.exports = {
  // eslint-config-soybeanjs-vue@0.2.4 的 index 自引用会死循环，改用 soybeanjs/vue
  extends: ['soybeanjs/vue'],
  ignorePatterns: ['dist', 'node_modules', 'src-tauri', 'md/**', 'database/**', 'backups', 'logs', '*.md'],
  settings: {
    'import/core-modules': ['uno.css', '~icons/*', 'virtual:svg-icons-register']
  },
  globals: {
    // 项目 ambient / 全局类型（declare namespace / 第三方 SDK）
    AuthRoute: 'readonly',
    UserManagement: 'readonly',
    RoleManagement: 'readonly',
    ApiUserManagement: 'readonly',
    ApiRoleManagement: 'readonly',
    GlobalMenuOption: 'readonly',
    GlobalHeaderProps: 'readonly',
    NaiveUI: 'readonly',
    EnumType: 'readonly',
    Message: 'readonly',
    Expose: 'readonly',
    NodeJS: 'readonly',
    echarts: 'readonly',
    BMap: 'readonly',
    AMap: 'readonly',
    TMap: 'readonly',
    PROJECT_BUILD_TIME: 'readonly'
  },
  rules: {
    // ESLint 8.57 + eslint-plugin-import@2.27 不兼容
    'import/namespace': 'off',
    // 现场业务代码里常见写法，先不挡提交
    'no-nested-ternary': 'off',
    complexity: 'off',
    'no-eq-null': 'off',
    eqeqeq: ['error', 'always', { null: 'ignore' }],
    'no-void': 'off',
    'no-continue': 'off',
    'require-atomic-updates': 'off',
    'import/order': [
      'error',
      {
        'newlines-between': 'never',
        groups: ['builtin', 'external', 'internal', 'parent', 'sibling', 'index'],
        pathGroups: [
          {
            pattern: 'vue',
            group: 'external',
            position: 'before'
          },
          {
            pattern: 'vue-router',
            group: 'external',
            position: 'before'
          },
          {
            pattern: 'pinia',
            group: 'external',
            position: 'before'
          },
          {
            pattern: 'naive-ui',
            group: 'external',
            position: 'before'
          },
          {
            pattern: '@/config',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/settings',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/enum',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/plugins',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/layouts',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/views',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/components',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/router',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/service',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/store',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/context',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/composables',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/hooks',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/utils',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/assets',
            group: 'internal',
            position: 'before'
          },
          {
            pattern: '@/**',
            group: 'internal',
            position: 'before'
          }
        ],
        pathGroupsExcludedImportTypes: ['vue', 'vue-router', 'pinia', 'naive-ui']
      }
    ]
  }
};
