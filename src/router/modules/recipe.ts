const recipe: AuthRoute.Route = {
  name: 'recipe',
  path: '/recipe',
  component: 'basic',
  children: [
    {
      name: 'recipe_list',
      path: '/recipe/list',
      component: 'self',
      meta: {
        title: '配方编辑器',
        requiresAuth: true,
        icon: 'mdi:playlist-edit'
      }
    }
  ],
  meta: {
    title: '配方管理',
    icon: 'mdi:flask-outline',
    order: 3
  }
};

export default recipe;
