const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        'index.html',
        { from: 'assets/catalog-nk.json', to: 'assets/catalog-nk.json' }
      ],
    })
  ],
  "experiments": {
    "asyncWebAssembly": true
  }
};
