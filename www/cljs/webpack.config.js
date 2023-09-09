const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: './build/index.js',
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin({patterns: ['build/index.html', 'build/main.js']})
  ],
  devServer: {
    hot: false,
    liveReload: false,
  },
  "experiments": {
    "asyncWebAssembly": true
  }
};
