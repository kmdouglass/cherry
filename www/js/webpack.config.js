const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./src/index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bundle.js",
  },
  mode: "development",
  module: {
    rules: [
      {
        test: /\.js|\.jsx$/,
        exclude: "/node_modules/",
        use: {
          loader: "babel-loader",
          options:{
              presets:[
                "@babel/preset-env",
                "@babel/preset-react"
              ]
          }
        }
      }
    ]
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        { from: "src/index.html", to: "index.html" },
        { from: "assets/catalog-nk.json", to: "assets/catalog-nk.json" }
      ],
    })
  ],
  "experiments": {
    "asyncWebAssembly": true
  }
};
