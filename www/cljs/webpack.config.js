const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: './public/js/index.js',
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin({patterns: ['public/css', 'public/index.html', 'public/js']})
  ],
  devServer: {
    hot: false,
    liveReload: false,
  },
  "experiments": {
    "asyncWebAssembly": true
  }
};
