import VueMacros from 'unplugin-vue-macros/vite';
import Icons from 'unplugin-icons/vite';
import IconsResolver from 'unplugin-icons/resolver';
import Components from 'unplugin-vue-components/vite';
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers';
import { FileSystemIconLoader } from 'unplugin-icons/loaders';
import { createSvgIconsPlugin } from 'vite-plugin-svg-icons';
import { getSrcPath } from '../utils';

export default function unplugin(viteEnv: ImportMetaEnv) {
  const iconPrefix = viteEnv.VITE_ICON_PREFFIX || 'icon';
  const localIconPrefix = viteEnv.VITE_ICON_LOCAL_PREFFIX || `${iconPrefix}-local`;

  const srcPath = getSrcPath();
  const localIconPath = `${srcPath}/assets/svg-icon`;

  /** 本地svg图标集合名称 */
  const collectionName = localIconPrefix.replace(`${iconPrefix}-`, '');

  return [
    VueMacros(),
    Icons({
      compiler: 'vue3',
      autoInstall: false,
      customCollections: {
        [collectionName]: FileSystemIconLoader(localIconPath)
      },
      scale: 1,
      defaultClass: 'inline-block'
    }),
    Components({
      dts: 'src/typings/components.d.ts',
      types: [{ from: 'vue-router', names: ['RouterLink', 'RouterView'] }],
      resolvers: [
        NaiveUiResolver(),
        IconsResolver({ customCollections: [collectionName], componentPrefix: iconPrefix })
      ]
    }),
    createSvgIconsPlugin({
      iconDirs: [localIconPath],
      symbolId: `${localIconPrefix}-[dir]-[name]`,
      inject: 'body-last',
      customDomId: '__SVG_ICON_LOCAL__'
    })
  ];
}
