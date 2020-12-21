const { default: merge } = require('webpack-merge');

module.exports = merge(require('./base.config'), {
  mode: 'development',
  devServer: {
    host: 'localhost',
    port: 8080,
  },
});
