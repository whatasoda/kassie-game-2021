const { default: merge } = require('webpack-merge');
const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const HTMLWebpackPlugin = require('html-webpack-plugin');

const __rootdir = path.resolve(__dirname, '../../');

module.exports = merge({
  entry: path.resolve(__rootdir, 'src-ts/index.ts'),
  output: {
    path: path.resolve(__rootdir, 'dist'),
  },
  experiments: {
    asyncWebAssembly: true,
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: __rootdir,
    }),
    new HTMLWebpackPlugin({
      template: path.resolve(__rootdir, 'template.html'),
    }),
  ],
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: [{ loader: 'ts-loader', options: { transpileOnly: true } }],
      },
    ],
  },
});
